import random
from entities import Plant, Rabbit, Deer, Fox, Wolf, Animal


class Ecosystem:
    """Manages the entire ecosystem simulation"""

    def __init__(self, world_width, world_height, initial_populations=None):
        self.world_width = world_width
        self.world_height = world_height
        self.time_step = 0

        # Entity collections
        self.plants = []
        self.rabbits = []
        self.deer = []
        self.foxes = []
        self.wolves = []

        # Statistics tracking
        self.history = {
            'time': [],
            'plants': [],
            'rabbits': [],
            'deer': [],
            'foxes': [],
            'wolves': [],
            'total_births': 0,
            'total_deaths': 0,
            'total_hunts': 0
        }

        # Population limits to prevent overpopulation
        self.max_plants = 800
        self.max_animals = 500

        # Initialize populations
        if initial_populations is None:
            initial_populations = {
                'plants': 300,
                'rabbits': 40,
                'deer': 20,
                'foxes': 8,
                'wolves': 3
            }

        self._initialize_populations(initial_populations)

    def _initialize_populations(self, populations):
        """Create initial entities"""
        # Plants
        for _ in range(populations.get('plants', 0)):
            x = random.uniform(0, self.world_width)
            y = random.uniform(0, self.world_height)
            self.plants.append(Plant(x, y, self.world_width, self.world_height))

        # Rabbits
        for _ in range(populations.get('rabbits', 0)):
            x = random.uniform(0, self.world_width)
            y = random.uniform(0, self.world_height)
            self.rabbits.append(Rabbit(x, y, self.world_width, self.world_height))

        # Deer
        for _ in range(populations.get('deer', 0)):
            x = random.uniform(0, self.world_width)
            y = random.uniform(0, self.world_height)
            self.deer.append(Deer(x, y, self.world_width, self.world_height))

        # Foxes
        for _ in range(populations.get('foxes', 0)):
            x = random.uniform(0, self.world_width)
            y = random.uniform(0, self.world_height)
            self.foxes.append(Fox(x, y, self.world_width, self.world_height))

        # Wolves
        for _ in range(populations.get('wolves', 0)):
            x = random.uniform(0, self.world_width)
            y = random.uniform(0, self.world_height)
            self.wolves.append(Wolf(x, y, self.world_width, self.world_height))

    def step(self):
        """Advance the simulation by one time step"""
        self.time_step += 1

        # Get all collections
        all_herbivores = self.rabbits + self.deer
        all_carnivores = self.foxes + self.wolves

        # 1. Plant phase - grow and spread
        self._update_plants()

        # 2. Herbivore phase - behavior, eating, aging
        for rabbit in self.rabbits[:]:
            if rabbit.alive:
                rabbit.behave(self.plants, all_carnivores, self.rabbits)
                rabbit.update_age()
                if rabbit.reproduction_cooldown > 0:
                    rabbit.reproduction_cooldown -= 1

        for deer in self.deer[:]:
            if deer.alive:
                deer.behave(self.plants, all_carnivores, self.deer)
                deer.update_age()
                if deer.reproduction_cooldown > 0:
                    deer.reproduction_cooldown -= 1

        # 3. Carnivore phase - behavior, hunting, aging
        for fox in self.foxes[:]:
            if fox.alive:
                fox.behave(all_herbivores, self.foxes)
                fox.update_age()
                if fox.reproduction_cooldown > 0:
                    fox.reproduction_cooldown -= 1

        for wolf in self.wolves[:]:
            if wolf.alive:
                wolf.behave(all_herbivores, self.wolves)
                wolf.update_age()
                if wolf.reproduction_cooldown > 0:
                    wolf.reproduction_cooldown -= 1

        # 4. Reproduction phase
        self._handle_reproduction()

        # 5. Cleanup dead entities
        self._cleanup_dead()

        # 6. Record statistics
        self._record_statistics()

    def _update_plants(self):
        """Update plant growth and spreading"""
        # Grow existing plants
        for plant in self.plants:
            if plant.alive:
                plant.grow()

                # Attempt to spread
                new_plant = plant.spread(self.plants, self.max_plants)
                if new_plant:
                    self.plants.append(new_plant)

    def _handle_reproduction(self):
        """Handle mating and reproduction for all species"""
        # Check total animal count
        total_animals = len(self.rabbits) + len(self.deer) + len(self.foxes) + len(self.wolves)

        if total_animals >= self.max_animals:
            return  # Population limit reached

        # Rabbit reproduction
        self._reproduce_species(self.rabbits, Rabbit)

        # Deer reproduction
        self._reproduce_species(self.deer, Deer)

        # Fox reproduction
        self._reproduce_species(self.foxes, Fox)

        # Wolf reproduction
        self._reproduce_species(self.wolves, Wolf)

    def _reproduce_species(self, population, species_class):
        """Handle reproduction for a specific species"""
        # Find potential mating pairs
        ready_females = [a for a in population if a.alive and a.can_reproduce() and a.gender.value == 'female']
        ready_males = [a for a in population if a.alive and a.can_reproduce() and a.gender.value == 'male']

        if not ready_females or not ready_males:
            return

        # Check each female for nearby males
        for female in ready_females:
            for male in ready_males:
                if female.distance_to(male) < 3.0:  # Must be close to mate
                    # Both parents lose energy
                    female.consume_energy(female.reproduction_energy_cost)
                    male.consume_energy(male.reproduction_energy_cost)

                    # Set cooldowns
                    female.reproduction_cooldown = 50
                    male.reproduction_cooldown = 30

                    # Create offspring (potentially multiple)
                    num_offspring = getattr(female, 'litter_size', 1)

                    for _ in range(num_offspring):
                        # Inherit genetics from parents (average with mutation)
                        offspring_genetics = {}
                        for key in female.genetics:
                            parent_avg = (female.genetics[key] + male.genetics[key]) / 2
                            offspring_genetics[key] = max(0.5, min(1.5, parent_avg + random.gauss(0, 0.1)))

                        # Spawn near mother
                        offset_x = random.uniform(-2, 2)
                        offset_y = random.uniform(-2, 2)
                        x = max(0, min(female.x + offset_x, self.world_width))
                        y = max(0, min(female.y + offset_y, self.world_height))

                        offspring = species_class(x, y, self.world_width, self.world_height, offspring_genetics)
                        population.append(offspring)
                        self.history['total_births'] += 1

                    break  # Female can only mate once per opportunity

    def _cleanup_dead(self):
        """Remove dead entities and update death statistics"""
        initial_counts = {
            'plants': len(self.plants),
            'rabbits': len(self.rabbits),
            'deer': len(self.deer),
            'foxes': len(self.foxes),
            'wolves': len(self.wolves)
        }

        self.plants = [p for p in self.plants if p.alive]
        self.rabbits = [r for r in self.rabbits if r.alive]
        self.deer = [d for d in self.deer if d.alive]
        self.foxes = [f for f in self.foxes if f.alive]
        self.wolves = [w for w in self.wolves if w.alive]

        # Count deaths
        deaths = sum([
            initial_counts['plants'] - len(self.plants),
            initial_counts['rabbits'] - len(self.rabbits),
            initial_counts['deer'] - len(self.deer),
            initial_counts['foxes'] - len(self.foxes),
            initial_counts['wolves'] - len(self.wolves)
        ])
        self.history['total_deaths'] += deaths

    def _record_statistics(self):
        """Record current population statistics"""
        self.history['time'].append(self.time_step)
        self.history['plants'].append(len(self.plants))
        self.history['rabbits'].append(len(self.rabbits))
        self.history['deer'].append(len(self.deer))
        self.history['foxes'].append(len(self.foxes))
        self.history['wolves'].append(len(self.wolves))

    def get_all_entities(self):
        """Return all entities organized by type"""
        return {
            'plants': self.plants,
            'rabbits': self.rabbits,
            'deer': self.deer,
            'foxes': self.foxes,
            'wolves': self.wolves
        }

    def get_populations(self):
        """Return current population counts"""
        return {
            'plants': len(self.plants),
            'rabbits': len(self.rabbits),
            'deer': len(self.deer),
            'foxes': len(self.foxes),
            'wolves': len(self.wolves),
            'total_herbivores': len(self.rabbits) + len(self.deer),
            'total_carnivores': len(self.foxes) + len(self.wolves)
        }

    def get_statistics(self):
        """Return detailed statistics about the ecosystem"""
        populations = self.get_populations()

        # Calculate average energy and age for living animals
        stats = {
            'populations': populations,
            'time_step': self.time_step,
            'total_births': self.history['total_births'],
            'total_deaths': self.history['total_deaths']
        }

        # Add average stats for each species
        for species_name, population in [
            ('rabbits', self.rabbits),
            ('deer', self.deer),
            ('foxes', self.foxes),
            ('wolves', self.wolves)
        ]:
            if population:
                avg_energy = sum(a.energy for a in population) / len(population)
                avg_age = sum(a.age for a in population) / len(population)
                stats[f'{species_name}_avg_energy'] = avg_energy
                stats[f'{species_name}_avg_age'] = avg_age
            else:
                stats[f'{species_name}_avg_energy'] = 0
                stats[f'{species_name}_avg_age'] = 0

        return stats

    def add_entity(self, entity_type, x=None, y=None):
        """Manually add an entity to the ecosystem"""
        if x is None:
            x = random.uniform(0, self.world_width)
        if y is None:
            y = random.uniform(0, self.world_height)

        entity_map = {
            'plant': (Plant, self.plants),
            'rabbit': (Rabbit, self.rabbits),
            'deer': (Deer, self.deer),
            'fox': (Fox, self.foxes),
            'wolf': (Wolf, self.wolves)
        }

        if entity_type.lower() in entity_map:
            entity_class, population = entity_map[entity_type.lower()]
            new_entity = entity_class(x, y, self.world_width, self.world_height)
            population.append(new_entity)
            return new_entity

        return None
