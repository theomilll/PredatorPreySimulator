# 🌍 Advanced Ecosystem Simulator - Predator-Prey Dynamics

A comprehensive, realistic ecosystem simulation featuring multiple species, energy-based survival, genetic inheritance, and complex behavioral AI.

## 🎯 Overview

This simulator models a complete food chain ecosystem with plants, herbivores, and carnivores. Each entity has realistic behaviors including:
- Energy-based survival and reproduction
- Genetic traits passed to offspring
- Intelligent behavioral AI (hunting, fleeing, seeking food/mates)
- Age and natural lifespan
- Gender-based reproduction requiring mates

## 🌟 Features

### Multi-Species Ecosystem
- **Plants (🌱)**: Primary producers that grow and spread naturally
- **Rabbits (🐰)**: Small, fast herbivores with high reproduction rate
- **Deer (🦌)**: Large herbivores with good vision and endurance
- **Foxes (🦊)**: Agile predators that hunt rabbits
- **Wolves (🐺)**: Apex predators that hunt both deer and rabbits

### Realistic Mechanics

#### Energy System
- All animals require energy to survive
- Energy depletes through movement and activities
- Eating restores energy
- Starvation leads to death
- Energy efficiency varies by genetics

#### Genetics & Evolution
- Four inheritable traits: speed, vision range, size, energy efficiency
- Traits passed from parents to offspring with random mutations
- Natural selection favors advantageous traits

#### Reproduction
- Requires two animals of opposite gender
- Both parents must have sufficient energy
- Proximity-based mating (animals must be close)
- Cooldown period after reproduction
- Multiple offspring possible (litter size varies by species)
- Genetic inheritance with variation

#### Behavioral AI
Each animal exhibits complex behaviors with priority-based decision making:

**Herbivores:**
1. Flee from nearby predators (highest priority)
2. Seek food when hungry
3. Seek mates when well-fed and mature
4. Wander when no immediate needs

**Carnivores:**
1. Hunt prey when hungry
2. Seek mates when well-fed
3. Wander when no immediate needs

#### Vision & Perception
- Each species has unique vision range
- Vision affected by genetics
- Animals detect food, predators, and mates within range
- Hunting success depends on predator/prey speed ratio

#### Age & Lifespan
- Animals age each time step
- Species have different natural lifespans
- Death occurs from starvation, predation, or old age

### Advanced Visualization

#### Main Canvas
- Dark background for better contrast
- Different colors and shapes per species
- Herbivores: circles, Carnivores: triangles
- Color changes based on behavior state
- Size varies by species and genetics

#### Visual Indicators
- **Energy Bars**: Color-coded health indicators (green → yellow → red)
- **Gender Markers**: Blue (male) / Pink (female) dots
- **Vision Ranges**: Optional display of detection radius
- **Target Lines**: Show hunting/fleeing/seeking relationships
- **Plant Maturity**: Darker green = more nutritious

### Interactive Controls

- **▶/⏸ Play/Pause**: Control simulation flow
- **Speed Slider**: Adjust simulation speed (10-200ms per step)
- **Add Entities**: Dynamically add plants or animals
- **View Options**: Toggle energy bars and vision ranges

### Statistics Panel

Real-time tracking of:
- Population counts for all species
- Total births and deaths
- Average energy levels per species
- Average age per species
- Herbivore/carnivore ratios

### Population Graphs

Live updating graphs showing:
- Population dynamics over time
- All species on one plot
- Color-coded with legend
- Grid for easy reading

## 📋 Requirements

### Python Packages
```bash
pip install tkinter matplotlib
```

**Note**: `tkinter` usually comes pre-installed with Python. If not:
- **Ubuntu/Debian**: `sudo apt-get install python3-tk`
- **macOS**: Included with Python
- **Windows**: Included with Python

## 🚀 Running the Simulation

### Main GUI Application
```bash
python simulation_gui.py
```

This launches the full interactive GUI with controls, statistics, and live graphs.

### Standalone Graphing
```bash
python graphing.py
```

This runs a matplotlib-based animation showing only population graphs.

## 🎮 Usage Guide

### Getting Started
1. Launch the simulation: `python simulation_gui.py`
2. Observe the ecosystem dynamics in real-time
3. Watch as populations rise and fall based on predator-prey relationships

### Controls
- **Pause/Play**: Freeze the simulation to examine details
- **Speed**: Slow down to watch behaviors or speed up for long-term trends
- **Add Entities**: Boost populations to test ecosystem responses

### Interesting Things to Try

1. **Predator Removal**: Add many plants and herbivores, remove predators
   - Watch herbivore population explode then crash from starvation

2. **Apex Predator Introduction**: Add wolves to a stable fox-rabbit ecosystem
   - Observe how wolves outcompete foxes

3. **Resource Scarcity**: Remove plants and watch the cascade effect
   - Herbivores starve → Predators starve

4. **Genetic Drift**: Run for 1000+ steps and observe trait evolution
   - Faster prey survive better → faster predators needed

## 🏗️ Architecture

### File Structure
```
PredatorPreySimulator/
├── entities.py          # Entity classes (Plant, Animal, species)
├── ecosystem.py         # Ecosystem manager and simulation logic
├── simulation_gui.py    # Main GUI application
├── graphing.py          # Standalone graphing utilities
└── README.md           # This file
```

### Class Hierarchy
```
Entity (base)
├── Plant
└── Animal
    ├── Herbivore
    │   ├── Rabbit
    │   └── Deer
    └── Carnivore
        ├── Fox
        └── Wolf
```

## 📊 Species Characteristics

| Species | Speed | Vision | Max Energy | Max Age | Diet | Litter Size |
|---------|-------|--------|------------|---------|------|-------------|
| 🌱 Plant | - | - | - | ∞ | Sunlight | 1 (spread) |
| 🐰 Rabbit | 1.2 | 15 | 80 | 150 | Plants | 2-4 |
| 🦌 Deer | 1.5 | 25 | 150 | 300 | Plants | 1 |
| 🦊 Fox | 1.3 | 20 | 100 | 250 | Rabbits | 2-3 |
| 🐺 Wolf | 1.4 | 30 | 180 | 350 | Deer, Rabbits | 2-4 |

## 🔬 Simulation Parameters

### Tunable Constants (in entities.py and ecosystem.py)

**Energy Costs:**
- Wandering: 0.05-0.08 per step
- Moving to food: 0.15 per step
- Fleeing: 0.3 per step
- Hunting: 0.25 per step

**Reproduction:**
- Proximity required: 3.0 units
- Female cooldown: 50 steps
- Male cooldown: 30 steps
- Minimum age varies by species
- Minimum energy varies by species

**Population Limits:**
- Max plants: 800
- Max animals (total): 500

## 🎨 Visual Guide

### Colors
- **Plants**: Light green → Dark green (maturity)
- **Rabbits**: Forest green (normal), Light green (fleeing)
- **Deer**: Saddle brown (normal), Tan (fleeing)
- **Foxes**: Dark orange (normal), Orange-red (hunting)
- **Wolves**: Gray (normal), Dark red (hunting)

### Shapes
- **Plants**: Small circles (size = maturity)
- **Herbivores**: Circles (size = species + genetics)
- **Carnivores**: Triangles (size = species + genetics)

## 🔍 Advanced Features

### Genetic System
Each animal has 4 genes (0.5x - 1.5x multipliers):
- `speed`: Movement and chase/flee effectiveness
- `vision_range`: Detection distance multiplier
- `size`: Affects visual size and predator nutrition
- `efficiency`: Reduces energy consumption

### Behavior States
Entities can be in different states (visible via color changes):
- `WANDERING`: Random movement
- `SEEKING_FOOD`: Moving toward food source
- `FLEEING`: Escaping from predator
- `HUNTING`: Chasing prey
- `SEEKING_MATE`: Moving toward potential mate
- `RESTING`: Stationary after eating

## 🐛 Known Limitations

1. No spatial optimization - may slow with 500+ entities
2. Vision is circular (no obstacles/line-of-sight blocking)
3. No herding/pack behavior (yet)
4. No seasons or environmental variation (yet)

## 🚀 Future Enhancements

Potential additions:
- Seasonal cycles affecting plant growth
- Water sources as requirement
- Pack hunting for wolves
- Herding behavior for herbivores
- Terrain types (forests, plains)
- Disease/parasites
- Migration patterns
- Camouflage genetics
- Parent-offspring bonding

## 📝 License

This project is open source and available for educational and research purposes.

## 🤝 Contributing

Feel free to fork, modify, and experiment! Interesting features to add:
- More species (birds, fish, insects)
- Environmental disasters
- Human impact simulation
- Climate change effects
- Ecosystem balance scoring

## 📚 Educational Value

This simulation demonstrates:
- **Ecology**: Food chains, population dynamics, carrying capacity
- **Evolution**: Natural selection, genetic drift
- **Systems Thinking**: Feedback loops, emergent behavior
- **Game Theory**: Optimal foraging, survival strategies
- **Computer Science**: OOP, AI behavior trees, simulation design

Enjoy exploring the complex dynamics of predator-prey relationships! 🌍🦊🐰
