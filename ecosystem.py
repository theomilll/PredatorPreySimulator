from entities import Predator, Prey


class Ecosystem:
    def __init__(self, num_prey, num_predators, x_bound, y_bound):
        self.x_bound = x_bound
        self.y_bound = y_bound
        self.prey = [Prey(x_bound, y_bound) for _ in range(num_prey)]
        self.predators = [Predator(x_bound, y_bound) for _ in range(num_predators)]

    def step(self):
        for p in self.prey[:]:
            new_prey = p.reproduce(self.x_bound, self.y_bound)
            if new_prey:
                self.prey.append(new_prey)
            p.move(self.x_bound, self.y_bound)

        for predator in self.predators:
            ate = predator.eat(self.prey)
            if not ate:
                predator.starve()
            predator.move(self.x_bound, self.y_bound)

        self.prey = [p for p in self.prey if p.alive]
        self.predators = [predator for predator in self.predators if predator.alive]
