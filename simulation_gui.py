import tkinter as tk
from tkinter import ttk

from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure

from ecosystem import Ecosystem
from entities import Predator, Prey


class SimulationApp:
    def __init__(self, master, ecosystem, size=600):
        self.master = master
        self.ecosystem = ecosystem
        self.size = size

        # Setting up the simulation canvas
        self.simulation_frame = ttk.Frame(master)
        self.simulation_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        self.canvas = tk.Canvas(self.simulation_frame, width=size, height=size)
        self.canvas.pack()

        # Setting up the plotting area
        self.figure = Figure(figsize=(5, 4), dpi=100)
        self.plot = self.figure.add_subplot(1, 1, 1)

        self.canvas_fig = FigureCanvasTkAgg(self.figure, master)
        self.canvas_fig.draw()
        self.canvas_fig.get_tk_widget().pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)

        self.prey_population = []
        self.predator_population = []
        self.time_steps = []

    def draw_entity(self, entity):
        color = 'green' if isinstance(entity, Prey) else 'red'
        x1, y1 = entity.x * (self.size / self.ecosystem.x_bound), entity.y * (self.size / self.ecosystem.y_bound)
        x2, y2 = x1 + 10, y1 + 10
        self.canvas.create_rectangle(x1, y1, x2, y2, fill=color)

    def update_graph(self):
        prey_pop, pred_pop = len(self.ecosystem.prey), len(self.ecosystem.predators)
        self.prey_population.append(prey_pop)
        self.predator_population.append(pred_pop)
        self.time_steps.append(len(self.time_steps))

        self.plot.clear()
        self.plot.plot(self.time_steps, self.prey_population, label="Prey")
        self.plot.plot(self.time_steps, self.predator_population, label="Predators")
        self.plot.set_xlabel('Time Step')
        self.plot.set_ylabel('Population')
        self.plot.legend(loc="upper right")
        self.canvas_fig.draw()

    def update(self):
        self.canvas.delete("all")
        for entity in self.ecosystem.prey + self.ecosystem.predators:
            self.draw_entity(entity)
        self.ecosystem.step()
        self.update_graph()  # Update the graph along with the ecosystem
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
