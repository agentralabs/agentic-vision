# Framework Integration Guide

Cortex integrates with popular AI agent frameworks. All integrations support the full v1.0 workflow: map, compile, query (WQL), track (temporal), and act.

## LangChain

```python
from cortex_langchain import CortexMapTool, CortexQueryTool, CortexActTool, CortexCompileTool, CortexWqlTool
from langchain.agents import initialize_agent

tools = [CortexMapTool(), CortexCompileTool(), CortexWqlTool(), CortexQueryTool(), CortexActTool()]
agent = initialize_agent(tools, llm, agent="zero-shot-react-description")

result = agent.run("Map amazon.com, compile it, and find products under $50 using WQL")
```

## CrewAI

```python
from cortex_crewai import CortexWebCartographer
from crewai import Agent, Task, Crew

researcher = Agent(
    role="Web Researcher",
    tools=[CortexWebCartographer()],
)

task = Task(
    description="Map example.com and find all documentation pages",
    agent=researcher,
)

crew = Crew(agents=[researcher], tasks=[task])
crew.kickoff()
```

## OpenClaw

```json
{
  "skills": ["cortex_map", "cortex_navigate"],
  "task": "Map shop.example.com, find products with rating > 4.5"
}
```

```python
from openclaw import Agent
from cortex_openclaw.skills import cortex_map, cortex_navigate

agent = Agent(skills=[cortex_map, cortex_navigate])
result = agent.run("Find the cheapest laptop on shop.example.com")
```

## MCP (Claude, Cursor, Windsurf)

Cortex provides a native MCP server. Use `cortex plug` to auto-inject it into supported agents:

```bash
cortex plug              # Auto-discover and inject into all agents
cortex plug --list       # See detected agents
cortex plug --status     # Check which agents have Cortex
```

The MCP server exposes map, compile, wql, query, pathfind, perceive, and act as tools.

## Further Reading

- [Web Compiler Guide](web-compiler.md) — compile maps into typed APIs
- [WQL Guide](wql.md) — SQL-like queries across web data
- [Temporal Intelligence](temporal.md) — track changes and predict values
- [Collective Graph](collective-graph.md) — share maps between agents
