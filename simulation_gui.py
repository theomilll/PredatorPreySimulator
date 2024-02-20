import tkinter as tk

from ecosystem import Ecosystem
from entities import Predator, Prey


class SimulationApp:
    def __init__(self, master, ecosystem, size=600):
        self.master = master
        self.ecosystem = ecosystem
        self.size = size
        self.canvas = tk.Canvas(master, width=size, height=size)
        self.canvas.pack()

    def draw_entity(self, entity):
        color = 'green' if isinstance(entity, Prey) else 'red'
        x1, y1 = entity.x * (self.size / self.ecosystem.x_bound), entity.y * (self.size / self.ecosystem.y_bound)
        x2, y2 = x1 + 10, y1 + 10
        self.canvas.create_rectangle(x1, y1, x2, y2, fill=color)

    def update(self):
        self.canvas.delete("all")
        for entity in self.ecosystem.prey + self.ecosystem.predators:
            self.draw_entity(entity)
        self.ecosystem.step()
        self.master.after(100, self.update)

def main():
    root = tk.Tk()
    root.title("Predator-Prey Simulation")
    size = 600
    x_bound, y_bound = size // 10, size // 10
    ecosystem = Ecosystem(50, 10, x_bound, y_bound)
    app = SimulationApp(root, ecosystem, size)
    app.update()
    root.mainloop()

if __name__ == "__main__":
    main()
