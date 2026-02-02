# Kinhin - Claude Code Project Instructions

## Project Overview

**Kinhin** (経行 / 禅歩) is the practice of walking meditation in Zen Buddhism, serving as active transition between periods of zazen (seated meditation).

**Philosophy**: Like walking meditation with deliberate steps, Kinhin guides developers through TDD with mindful progression - red, green, refactor.


## Structure

```
kinhin/
├── kinhin/                   # Bundle (Gradient pattern)
│   ├── spec/
│   │   ├── tdd/              # TDD methodology
│   │   └── python-tdd/       # Python-specific TDD
│   ├── context/
│   │   ├── guides/           # TDD workflow guides
│   │   ├── examples/         # Test templates
│   │   └── checklists/       # TDD checklists
│   └── prompts/
├── commands/
├── skills/
└── docs/
```


## Commands

| Command | Purpose |
|---------|---------|
| `/kinhin:load` | Load TDD principles |
| `/kinhin:load-python` | Load Python-specific TDD |


## Related Plugins

- **Zazen**: Code quality (naming, structure, zen, python)
- **Arche**: Behavioral principles for Claude Code
- **Gradient**: Plugin architecture


## Origin

Kinhin was extracted from Zazen to separate concerns:
- **Zazen**: Code quality principles (151 test cases)
- **Kinhin**: TDD practices (45 test cases)
