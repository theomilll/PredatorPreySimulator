import random
import math
from enum import Enum


class Gender(Enum):
    MALE = "male"
    FEMALE = "female"


class BehaviorState(Enum):
    WANDERING = "wandering"
    SEEKING_FOOD = "seeking_food"
    FLEEING = "fleeing"
    HUNTING = "hunting"
    SEEKING_MATE = "seeking_mate"
    RESTING = "resting"


class Entity:
    """Base class for all entities in the ecosystem"""

    def __init__(self, x, y, world_width, world_height):
        self.x = x
        self.y = y
        self.world_width = world_width
        self.world_height = world_height
        self.alive = True
        self.age = 0

    def distance_to(self, other):
        """Calculate Euclidean distance to another entity"""
        return math.sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)

    def move_towards(self, target_x, target_y, speed):
        """Move towards a target position"""
        dx = target_x - self.x
        dy = target_y - self.y
        distance = math.sqrt(dx ** 2 + dy ** 2)

        if distance > 0:
            # Normalize and apply speed
            self.x += (dx / distance) * speed
            self.y += (dy / distance) * speed

        self.clamp_position()

    def move_away_from(self, target_x, target_y, speed):
        """Move away from a target position"""
        dx = self.x - target_x
        dy = self.y - target_y
        distance = math.sqrt(dx ** 2 + dy ** 2)

        if distance > 0:
            # Normalize and apply speed
            self.x += (dx / distance) * speed
            self.y += (dy / distance) * speed

        self.clamp_position()

    def clamp_position(self):
        """Keep entity within world bounds"""
        self.x = max(0, min(self.x, self.world_width))
        self.y = max(0, min(self.y, self.world_height))


class Plant(Entity):
    """Plant entities that serve as food for herbivores"""

    def __init__(self, x, y, world_width, world_height):
        super().__init__(x, y, world_width, world_height)
        self.energy_value = 15  # Energy provided to herbivores
        self.maturity = random.randint(1, 5)  # Growth stage
        self.max_maturity = 10
        self.growth_rate = 0.05  # Chance to grow each step

    def grow(self):
        """Plants grow over time, becoming more nutritious"""
        if self.maturity < self.max_maturity:
            if random.random() < self.growth_rate:
                self.maturity += 1
                self.energy_value = 10 + self.maturity * 2

    def spread(self, plants, max_plants):
        """Plants can spread to nearby areas"""
        if len(plants) >= max_plants:
            return None

        if random.random() < 0.001 * (self.maturity / self.max_maturity):  # Mature plants spread more
            # Spawn new plant nearby
            spread_distance = 5
            new_x = self.x + random.uniform(-spread_distance, spread_distance)
            new_y = self.y + random.uniform(-spread_distance, spread_distance)

            new_x = max(0, min(new_x, self.world_width))
            new_y = max(0, min(new_y, self.world_height))

            return Plant(new_x, new_y, self.world_width, self.world_height)
        return None


class Animal(Entity):
    """Base class for all animals"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height)

        # Genetics - inheritable traits with some random variation
        if genetics is None:
            self.genetics = {
                'speed': 1.0,
                'vision_range': 1.0,
                'size': 1.0,
                'efficiency': 1.0  # Energy efficiency
            }
        else:
            # Inherit with mutation
            self.genetics = {
                key: max(0.5, min(1.5, val + random.gauss(0, 0.1)))
                for key, val in genetics.items()
            }

        # Basic attributes
        self.gender = random.choice([Gender.MALE, Gender.FEMALE])
        self.energy = 50
        self.max_energy = 100
        self.age = 0
        self.max_age = 100

        # Behavior
        self.state = BehaviorState.WANDERING
        self.target = None

        # Reproduction
        self.reproduction_cooldown = 0
        self.min_reproduction_age = 10
        self.reproduction_energy_cost = 30
        self.min_reproduction_energy = 50

    def update_age(self):
        """Age the animal and check for natural death"""
        self.age += 1
        if self.age >= self.max_age:
            self.alive = False

    def consume_energy(self, amount):
        """Reduce energy and check for starvation"""
        self.energy -= amount * (2.0 - self.genetics['efficiency'])
        if self.energy <= 0:
            self.alive = False

    def gain_energy(self, amount):
        """Gain energy from eating"""
        self.energy = min(self.max_energy, self.energy + amount)

    def can_reproduce(self):
        """Check if animal can reproduce"""
        return (self.age >= self.min_reproduction_age and
                self.energy >= self.min_reproduction_energy and
                self.reproduction_cooldown <= 0)

    def find_nearest(self, entities, max_range=None):
        """Find the nearest entity from a list"""
        if not entities:
            return None

        vision_range = self.base_vision_range * self.genetics['vision_range']
        nearest = None
        min_distance = float('inf')

        for entity in entities:
            if not entity.alive:
                continue
            distance = self.distance_to(entity)
            if distance < min_distance:
                if max_range is None or distance <= vision_range:
                    min_distance = distance
                    nearest = entity

        return nearest

    def wander(self):
        """Random wandering movement"""
        angle = random.uniform(0, 2 * math.pi)
        speed = self.base_speed * self.genetics['speed'] * 0.5  # Slower when wandering
        self.x += math.cos(angle) * speed
        self.y += math.sin(angle) * speed
        self.clamp_position()


class Herbivore(Animal):
    """Base class for herbivores"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)
        self.diet = "herbivore"

    def find_food(self, plants):
        """Look for nearby plants"""
        food = self.find_nearest(plants, max_range=True)
        if food:
            self.state = BehaviorState.SEEKING_FOOD
            self.target = food
        else:
            self.state = BehaviorState.WANDERING
            self.target = None
        return food

    def detect_predators(self, carnivores):
        """Check for nearby predators"""
        threat = self.find_nearest(carnivores, max_range=True)
        if threat and self.distance_to(threat) < self.flee_range:
            self.state = BehaviorState.FLEEING
            self.target = threat
            return True
        return False

    def eat(self, plants):
        """Attempt to eat a nearby plant"""
        for plant in plants[:]:
            if plant.alive and self.distance_to(plant) < 1.0:
                self.gain_energy(plant.energy_value)
                plant.alive = False
                self.state = BehaviorState.RESTING
                return True
        return False

    def behave(self, plants, carnivores, herbivores):
        """Main behavior logic for herbivores"""
        # Priority 1: Flee from predators
        if self.detect_predators(carnivores):
            speed = self.base_speed * self.genetics['speed'] * 1.5  # Fast when fleeing
            self.move_away_from(self.target.x, self.target.y, speed)
            self.consume_energy(0.3)  # Fleeing costs energy
            return

        # Priority 2: Seek food if hungry
        if self.energy < self.max_energy * 0.6:
            if self.find_food(plants):
                speed = self.base_speed * self.genetics['speed']
                self.move_towards(self.target.x, self.target.y, speed)
                self.consume_energy(0.15)
                self.eat(plants)
                return

        # Priority 3: Seek mate if ready
        if self.can_reproduce():
            potential_mates = [h for h in herbivores
                             if h.alive and h != self and
                             type(h) == type(self) and
                             h.gender != self.gender and
                             h.can_reproduce()]
            mate = self.find_nearest(potential_mates, max_range=True)
            if mate:
                self.state = BehaviorState.SEEKING_MATE
                self.target = mate
                speed = self.base_speed * self.genetics['speed'] * 0.8
                self.move_towards(mate.x, mate.y, speed)
                self.consume_energy(0.1)
                return

        # Default: Wander
        self.state = BehaviorState.WANDERING
        self.wander()
        self.consume_energy(0.05)


class Carnivore(Animal):
    """Base class for carnivores"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)
        self.diet = "carnivore"
        self.hunt_success_rate = 0.7

    def find_prey(self, herbivores):
        """Look for nearby prey"""
        # Filter for preferred prey
        suitable_prey = [h for h in herbivores
                        if h.alive and type(h).__name__ in self.prey_types]

        prey = self.find_nearest(suitable_prey, max_range=True)
        if prey:
            self.state = BehaviorState.HUNTING
            self.target = prey
        else:
            self.state = BehaviorState.WANDERING
            self.target = None
        return prey

    def hunt(self, herbivores):
        """Attempt to catch and eat prey"""
        for prey in herbivores[:]:
            if prey.alive and type(prey).__name__ in self.prey_types:
                if self.distance_to(prey) < 1.5:
                    # Hunt success depends on predator and prey traits
                    if random.random() < self.hunt_success_rate * (self.genetics['speed'] / prey.genetics['speed']):
                        self.gain_energy(prey.energy * 0.8 + prey.genetics['size'] * 20)
                        prey.alive = False
                        self.state = BehaviorState.RESTING
                        return True
        return False

    def behave(self, herbivores, carnivores):
        """Main behavior logic for carnivores"""
        # Priority 1: Hunt if hungry
        if self.energy < self.max_energy * 0.7:
            if self.find_prey(herbivores):
                speed = self.base_speed * self.genetics['speed'] * 1.3  # Fast when hunting
                self.move_towards(self.target.x, self.target.y, speed)
                self.consume_energy(0.25)  # Hunting costs energy
                self.hunt(herbivores)
                return

        # Priority 2: Seek mate if well-fed
        if self.can_reproduce() and self.energy > self.max_energy * 0.6:
            potential_mates = [c for c in carnivores
                             if c.alive and c != self and
                             type(c) == type(self) and
                             c.gender != self.gender and
                             c.can_reproduce()]
            mate = self.find_nearest(potential_mates, max_range=True)
            if mate:
                self.state = BehaviorState.SEEKING_MATE
                self.target = mate
                speed = self.base_speed * self.genetics['speed'] * 0.8
                self.move_towards(mate.x, mate.y, speed)
                self.consume_energy(0.1)
                return

        # Default: Wander
        self.state = BehaviorState.WANDERING
        self.wander()
        self.consume_energy(0.08)  # Carnivores use more energy at rest


# ============= SPECIFIC SPECIES =============

class Rabbit(Herbivore):
    """Small, fast herbivore with high reproduction rate"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)

        # Rabbit-specific traits
        self.base_speed = 1.2
        self.base_vision_range = 15
        self.flee_range = 20
        self.max_energy = 80
        self.energy = 50
        self.max_age = 150

        # Reproduction
        self.min_reproduction_age = 8
        self.reproduction_energy_cost = 20
        self.min_reproduction_energy = 35
        self.litter_size = random.randint(2, 4)  # Rabbits have multiple offspring

    def get_color(self):
        """Visual representation color"""
        if self.state == BehaviorState.FLEEING:
            return '#90EE90'  # Light green when fleeing
        return '#228B22'  # Forest green normally


class Deer(Herbivore):
    """Large herbivore with good vision"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)

        # Deer-specific traits
        self.base_speed = 1.5
        self.base_vision_range = 25
        self.flee_range = 25
        self.max_energy = 150
        self.energy = 80
        self.max_age = 300

        # Reproduction
        self.min_reproduction_age = 20
        self.reproduction_energy_cost = 40
        self.min_reproduction_energy = 60
        self.litter_size = 1  # Deer usually have one offspring

    def get_color(self):
        """Visual representation color"""
        if self.state == BehaviorState.FLEEING:
            return '#D2B48C'  # Tan when fleeing
        return '#8B4513'  # Saddle brown normally


class Fox(Carnivore):
    """Small, agile predator that hunts rabbits"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)

        # Fox-specific traits
        self.base_speed = 1.3
        self.base_vision_range = 20
        self.max_energy = 100
        self.energy = 60
        self.max_age = 250
        self.hunt_success_rate = 0.65

        # Diet
        self.prey_types = ['Rabbit']  # Foxes primarily hunt rabbits

        # Reproduction
        self.min_reproduction_age = 15
        self.reproduction_energy_cost = 35
        self.min_reproduction_energy = 55
        self.litter_size = random.randint(2, 3)

    def get_color(self):
        """Visual representation color"""
        if self.state == BehaviorState.HUNTING:
            return '#FF4500'  # Orange red when hunting
        return '#FF8C00'  # Dark orange normally


class Wolf(Carnivore):
    """Large apex predator that hunts deer and rabbits"""

    def __init__(self, x, y, world_width, world_height, genetics=None):
        super().__init__(x, y, world_width, world_height, genetics)

        # Wolf-specific traits
        self.base_speed = 1.4
        self.base_vision_range = 30
        self.max_energy = 180
        self.energy = 90
        self.max_age = 350
        self.hunt_success_rate = 0.75

        # Diet
        self.prey_types = ['Deer', 'Rabbit']  # Wolves hunt both

        # Reproduction
        self.min_reproduction_age = 25
        self.reproduction_energy_cost = 50
        self.min_reproduction_energy = 70
        self.litter_size = random.randint(2, 4)

    def get_color(self):
        """Visual representation color"""
        if self.state == BehaviorState.HUNTING:
            return '#8B0000'  # Dark red when hunting
        return '#696969'  # Dim gray normally
