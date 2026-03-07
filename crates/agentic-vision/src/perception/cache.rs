//! Intent Cache — eliminate redundant perception by caching extraction results.
//!
//! Cache key: url_normalized + intent_type + structural_hash + content_hash
//! Cache HIT: all four match -> return cached result, 0 tokens
//! Cache MISS: content hash changed -> re-extract, update cache
//! Cache INVALID: structural hash changed -> grammar drift, re-learn

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// TTL category for cached content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentVolatility {
    /// Dynamic content (prices, stock, news): 1-hour TTL.
    Dynamic,
    /// Semi-static content (reviews, descriptions): 24-hour TTL.
    SemiStatic,
    /// Static content (documentation, articles): 7-day TTL.
    Static,
    /// User-pinned: custom TTL.
    Pinned,
}

impl ContentVolatility {
    /// TTL in seconds for this volatility class.
    pub fn ttl_secs(self) -> u64 {
        match self {
            Self::Dynamic => 3600,           // 1 hour
            Self::SemiStatic => 86400,       // 24 hours
            Self::Static => 604800,          // 7 days
            Self::Pinned => u64::MAX,        // effectively permanent
        }
    }
}

/// Composite cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntentCacheKey {
    /// Normalized URL (lowercase, stripped of tracking params).
    pub url_normalized: String,
    /// Intent type string (e.g., "find_price", "read_content").
    pub intent_type: String,
    /// Structural hash of the page (for grammar drift detection).
    pub structural_hash: String,
    /// Content hash of the relevant region (for content change detection).
    pub content_hash: String,
}

impl IntentCacheKey {
    pub fn new(
        url: impl Into<String>,
        intent: impl Into<String>,
        structural_hash: impl Into<String>,
        content_hash: impl Into<String>,
    ) -> Self {
        Self {
            url_normalized: normalize_url(&url.into()),
            intent_type: intent.into(),
            structural_hash: structural_hash.into(),
            content_hash: content_hash.into(),
        }
    }
}

/// A cached extraction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCacheEntry {
    /// The extraction result (serialized as JSON value).
    pub result: serde_json::Value,

    /// When this entry was created.
    pub created_at: u64,

    /// When this entry expires (unix timestamp).
    pub expires_at: u64,

    /// How volatile the content is.
    pub volatility: ContentVolatility,

    /// Tokens saved by using this cache entry.
    pub tokens_saved: u32,

    /// Number of times this entry has been used.
    pub hit_count: u64,
}

impl IntentCacheEntry {
    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        let now = now_secs();
        now > self.expires_at
    }
}

/// Cache lookup result.
#[derive(Debug, Clone)]
pub enum CacheLookup {
    /// Exact match — return cached result, 0 tokens.
    Hit(IntentCacheEntry),
    /// Content changed — re-extract, update cache.
    ContentChanged,
    /// Structure changed — grammar drift, re-learn.
    StructuralDrift,
    /// No entry found.
    Miss,
}

/// A cache entry paired with its key (for serialization as a list).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheRecord {
    key: IntentCacheKey,
    entry: IntentCacheEntry,
}

/// The intent cache store.
#[derive(Debug, Clone, Default)]
pub struct IntentCache {
    entries: HashMap<IntentCacheKey, IntentCacheEntry>,
    /// Maximum entries before eviction.
    max_entries: usize,
    /// Total cache hits for statistics.
    total_hits: u64,
    /// Total cache misses for statistics.
    total_misses: u64,
}

// Custom serialization: HashMap<IntentCacheKey, _> can't be a JSON map key,
// so we serialize as a Vec of records.
impl Serialize for IntentCache {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let records: Vec<CacheRecord> = self
            .entries
            .iter()
            .map(|(k, v)| CacheRecord {
                key: k.clone(),
                entry: v.clone(),
            })
            .collect();
        let mut s = serializer.serialize_struct("IntentCache", 4)?;
        s.serialize_field("records", &records)?;
        s.serialize_field("max_entries", &self.max_entries)?;
        s.serialize_field("total_hits", &self.total_hits)?;
        s.serialize_field("total_misses", &self.total_misses)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for IntentCache {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct IntentCacheData {
            #[serde(default)]
            records: Vec<CacheRecord>,
            #[serde(default = "default_max_entries")]
            max_entries: usize,
            #[serde(default)]
            total_hits: u64,
            #[serde(default)]
            total_misses: u64,
        }
        fn default_max_entries() -> usize {
            10_000
        }
        let data = IntentCacheData::deserialize(deserializer)?;
        let entries: HashMap<IntentCacheKey, IntentCacheEntry> = data
            .records
            .into_iter()
            .map(|r| (r.key, r.entry))
            .collect();
        Ok(IntentCache {
            entries,
            max_entries: data.max_entries,
            total_hits: data.total_hits,
            total_misses: data.total_misses,
        })
    }
}

impl IntentCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 10_000,
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Create with a custom max entries limit.
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Look up a cache entry by key.
    pub fn lookup(&mut self, key: &IntentCacheKey) -> CacheLookup {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                self.total_misses += 1;
                return CacheLookup::Miss;
            }
            self.total_hits += 1;
            return CacheLookup::Hit(entry.clone());
        }

        // Check if we have an entry for the same URL+intent but different hashes
        let partial_match = self.entries.keys().find(|k| {
            k.url_normalized == key.url_normalized && k.intent_type == key.intent_type
        });

        if let Some(existing_key) = partial_match.cloned() {
            if existing_key.structural_hash != key.structural_hash {
                self.total_misses += 1;
                return CacheLookup::StructuralDrift;
            }
            if existing_key.content_hash != key.content_hash {
                self.total_misses += 1;
                return CacheLookup::ContentChanged;
            }
        }

        self.total_misses += 1;
        CacheLookup::Miss
    }

    /// Insert or update a cache entry.
    pub fn insert(
        &mut self,
        key: IntentCacheKey,
        result: serde_json::Value,
        volatility: ContentVolatility,
        tokens_saved: u32,
    ) {
        // Remove any existing entry for same url+intent first (before size check)
        let to_remove_dedup: Vec<_> = self
            .entries
            .keys()
            .filter(|k| k.url_normalized == key.url_normalized && k.intent_type == key.intent_type)
            .cloned()
            .collect();
        for k in to_remove_dedup {
            self.entries.remove(&k);
        }

        // Evict expired entries if at limit
        if self.entries.len() >= self.max_entries {
            self.evict_expired();
        }
        // If still at limit, evict oldest entries (at least 1)
        if self.entries.len() >= self.max_entries {
            self.evict_oldest((self.max_entries / 10).max(1));
        }

        let now = now_secs();
        let entry = IntentCacheEntry {
            result,
            created_at: now,
            expires_at: now + volatility.ttl_secs(),
            volatility,
            tokens_saved,
            hit_count: 0,
        };

        self.entries.insert(key, entry);
    }

    /// Invalidate all cache entries for a URL.
    pub fn invalidate_url(&mut self, url: &str) {
        let normalized = normalize_url(url);
        self.entries.retain(|k, _| k.url_normalized != normalized);
    }

    /// Invalidate all cache entries for a domain.
    pub fn invalidate_domain(&mut self, domain: &str) {
        self.entries
            .retain(|k, _| !k.url_normalized.contains(domain));
    }

    /// Remove all expired entries.
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, v| !v.is_expired());
    }

    /// Evict the N oldest entries.
    fn evict_oldest(&mut self, n: usize) {
        let mut entries: Vec<_> = self.entries.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
        entries.sort_by_key(|(_, ts)| *ts);
        for (key, _) in entries.into_iter().take(n) {
            self.entries.remove(&key);
        }
    }

    /// Cache hit rate.
    pub fn hit_rate(&self) -> f32 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f32 / total as f32
        }
    }

    /// Total tokens saved by cache hits.
    pub fn total_tokens_saved(&self) -> u64 {
        self.entries.values().map(|e| e.tokens_saved as u64 * e.hit_count).sum()
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.entries.len(),
            total_hits: self.total_hits,
            total_misses: self.total_misses,
            hit_rate: self.hit_rate(),
            total_tokens_saved: self.total_tokens_saved(),
        }
    }
}

/// Cache statistics for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f32,
    pub total_tokens_saved: u64,
}

/// Normalize a URL for cache key purposes.
fn normalize_url(url: &str) -> String {
    let url = url.trim().to_lowercase();
    // Strip trailing slash
    let url = url.strip_suffix('/').unwrap_or(&url);
    // Strip common tracking parameters
    if let Some(pos) = url.find('?') {
        let base = &url[..pos];
        let query = &url[pos + 1..];
        let filtered: Vec<&str> = query
            .split('&')
            .filter(|p| {
                let key = p.split('=').next().unwrap_or("");
                !matches!(
                    key,
                    "utm_source" | "utm_medium" | "utm_campaign" | "utm_term"
                        | "utm_content" | "fbclid" | "gclid" | "ref" | "ref_"
                )
            })
            .collect();
        if filtered.is_empty() {
            base.to_string()
        } else {
            format!("{}?{}", base, filtered.join("&"))
        }
    } else {
        url.to_string()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("https://example.com/"), "https://example.com");
        assert_eq!(
            normalize_url("https://example.com/page?utm_source=foo&id=123"),
            "https://example.com/page?id=123"
        );
        assert_eq!(
            normalize_url("HTTPS://Example.COM/Path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_cache_insert_and_lookup() {
        let mut cache = IntentCache::new();
        let key = IntentCacheKey::new("https://example.com", "find_price", "struct1", "content1");

        cache.insert(
            key.clone(),
            serde_json::json!({"price": "$1,299"}),
            ContentVolatility::Dynamic,
            2000,
        );

        match cache.lookup(&key) {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.result["price"], "$1,299");
            }
            other => panic!("Expected Hit, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn test_cache_content_changed() {
        let mut cache = IntentCache::new();
        let key1 = IntentCacheKey::new("https://example.com", "find_price", "struct1", "content1");
        cache.insert(
            key1,
            serde_json::json!({"price": "$1,299"}),
            ContentVolatility::Dynamic,
            2000,
        );

        // Same URL+intent, same structure, different content
        let key2 = IntentCacheKey::new("https://example.com", "find_price", "struct1", "content2");
        assert!(matches!(cache.lookup(&key2), CacheLookup::ContentChanged));
    }

    #[test]
    fn test_cache_structural_drift() {
        let mut cache = IntentCache::new();
        let key1 = IntentCacheKey::new("https://example.com", "find_price", "struct1", "content1");
        cache.insert(
            key1,
            serde_json::json!({"price": "$1,299"}),
            ContentVolatility::Dynamic,
            2000,
        );

        // Same URL+intent, different structure
        let key2 = IntentCacheKey::new("https://example.com", "find_price", "struct2", "content1");
        assert!(matches!(cache.lookup(&key2), CacheLookup::StructuralDrift));
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = IntentCache::new();
        let key = IntentCacheKey::new("https://example.com/page", "find_price", "s", "c");
        cache.insert(key, serde_json::json!({}), ContentVolatility::Static, 100);
        assert_eq!(cache.len(), 1);

        cache.invalidate_url("https://example.com/page");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = IntentCache::new();
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }
}
