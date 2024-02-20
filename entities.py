import random


class Entity:
    def __init__(self, x_bound, y_bound):
        self.x = random.randint(0, x_bound)
        self.y = random.randint(0, y_bound)
        self.alive = True

    def move(self, x_bound, y_bound):
        self.x += random.randint(-1, 1)
        self.y += random.randint(-1, 1)
        self.x = max(0, min(self.x, x_bound))
        self.y = max(0, min(self.y, y_bound))

class Prey(Entity):
    def reproduce(self, x_bound, y_bound):
        if random.random() < 0.1:
            return Prey(x_bound, y_bound)
        return None

class Predator(Entity):
    def __init__(self, x_bound, y_bound):
        super().__init__(x_bound, y_bound)
        self.time_since_last_meal = 0
        self.starvation_threshold = 10

    def eat(self, preys):
        for prey in preys:
            if abs(self.x - prey.x) <= 1 and abs(self.y - prey.y) <= 1 and prey.alive:
                self.time_since_last_meal = 0
                prey.alive = False
                return True
        return False

    def starve(self):
        self.time_since_last_meal += 1
        if self.time_since_last_meal > self.starvation_threshold:
            self.alive = False
