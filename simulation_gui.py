import tkinter as tk
from tkinter import ttk, Frame, Label, Button, Scale
import math

from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure

from ecosystem import Ecosystem
from entities import Plant, Rabbit, Deer, Fox, Wolf, BehaviorState


class SimulationGUI:
    """Enhanced GUI for the ecosystem simulation"""

    def __init__(self, master, world_size=100, canvas_size=800):
        self.master = master
        self.world_size = world_size
        self.canvas_size = canvas_size
        self.scale = canvas_size / world_size

        # Create ecosystem
        self.ecosystem = Ecosystem(world_size, world_size)

        # Simulation state
        self.running = True
        self.speed = 50  # milliseconds per step (lower = faster)
        self.show_vision = False
        self.show_energy = True

        # Setup UI
        self._setup_ui()

        # Start simulation loop
        self.update()

    def _setup_ui(self):
        """Setup all UI components"""
        # Main container
        main_frame = Frame(self.master)
        main_frame.pack(fill=tk.BOTH, expand=True)

        # Left side - simulation canvas
        left_frame = Frame(main_frame)
        left_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=5, pady=5)

        # Canvas for simulation
        self.canvas = tk.Canvas(left_frame, width=self.canvas_size, height=self.canvas_size,
                               bg='#1a1a1a', highlightthickness=0)
        self.canvas.pack()

        # Controls below canvas
        self._setup_controls(left_frame)

        # Right side - statistics and graphs
        right_frame = Frame(main_frame, width=400)
        right_frame.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True, padx=5, pady=5)

        self._setup_statistics(right_frame)
        self._setup_graphs(right_frame)

    def _setup_controls(self, parent):
        """Setup control buttons and sliders"""
        control_frame = Frame(parent)
        control_frame.pack(pady=10)

        # Play/Pause button
        self.play_pause_btn = Button(control_frame, text="⏸ Pause", command=self.toggle_pause,
                                     font=('Arial', 12), width=10)
        self.play_pause_btn.grid(row=0, column=0, padx=5)

        # Speed control
        Label(control_frame, text="Speed:", font=('Arial', 10)).grid(row=0, column=1, padx=5)
        self.speed_scale = Scale(control_frame, from_=10, to=200, orient=tk.HORIZONTAL,
                                command=self.change_speed, length=150)
        self.speed_scale.set(self.speed)
        self.speed_scale.grid(row=0, column=2, padx=5)

        # Add entity buttons
        Label(control_frame, text="Add:", font=('Arial', 10, 'bold')).grid(row=1, column=0, padx=5, pady=5)

        add_buttons = [
            ("🌱 Plants (10)", lambda: self.add_multiple_entities('plant', 10)),
            ("🐰 Rabbit", lambda: self.add_entity('rabbit')),
            ("🦌 Deer", lambda: self.add_entity('deer')),
            ("🦊 Fox", lambda: self.add_entity('fox')),
            ("🐺 Wolf", lambda: self.add_entity('wolf'))
        ]

        col = 1
        for text, command in add_buttons:
            btn = Button(control_frame, text=text, command=command, font=('Arial', 9))
            btn.grid(row=1, column=col, padx=2)
            col += 1

        # View options
        self.energy_var = tk.BooleanVar(value=True)
        self.vision_var = tk.BooleanVar(value=False)

        ttk.Checkbutton(control_frame, text="Show Energy Bars", variable=self.energy_var).grid(
            row=2, column=0, columnspan=2, sticky='w', pady=5)
        ttk.Checkbutton(control_frame, text="Show Vision Range", variable=self.vision_var).grid(
            row=2, column=2, columnspan=2, sticky='w', pady=5)

    def _setup_statistics(self, parent):
        """Setup statistics display panel"""
        stats_frame = Frame(parent, relief=tk.RIDGE, borderwidth=2)
        stats_frame.pack(fill=tk.BOTH, padx=5, pady=5)

        Label(stats_frame, text="📊 Ecosystem Statistics", font=('Arial', 14, 'bold')).pack(pady=5)

        self.stats_text = tk.Text(stats_frame, height=20, width=45, font=('Courier', 9),
                                 bg='#f0f0f0', relief=tk.FLAT)
        self.stats_text.pack(padx=5, pady=5)

    def _setup_graphs(self, parent):
        """Setup population graphs"""
        graph_frame = Frame(parent)
        graph_frame.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)

        # Create matplotlib figure
        self.figure = Figure(figsize=(6, 5), dpi=80)
        self.plot = self.figure.add_subplot(1, 1, 1)
        self.plot.set_facecolor('#f5f5f5')
        self.figure.patch.set_facecolor('white')

        self.canvas_fig = FigureCanvasTkAgg(self.figure, graph_frame)
        self.canvas_fig.draw()
        self.canvas_fig.get_tk_widget().pack(fill=tk.BOTH, expand=True)

    def draw_entities(self):
        """Draw all entities on the canvas"""
        self.canvas.delete("all")

        # Get current view settings
        show_energy = self.energy_var.get()
        show_vision = self.vision_var.get()

        entities = self.ecosystem.get_all_entities()

        # Draw plants (smallest, in background)
        for plant in entities['plants']:
            self._draw_plant(plant)

        # Draw herbivores
        for rabbit in entities['rabbits']:
            self._draw_animal(rabbit, show_energy, show_vision)

        for deer in entities['deer']:
            self._draw_animal(deer, show_energy, show_vision)

        # Draw carnivores (on top)
        for fox in entities['foxes']:
            self._draw_animal(fox, show_energy, show_vision)

        for wolf in entities['wolves']:
            self._draw_animal(wolf, show_energy, show_vision)

    def _draw_plant(self, plant):
        """Draw a plant entity"""
        x = plant.x * self.scale
        y = plant.y * self.scale

        # Size based on maturity
        size = 2 + (plant.maturity / plant.max_maturity) * 3

        # Color based on maturity (darker green = more mature)
        maturity_ratio = plant.maturity / plant.max_maturity
        green_value = int(100 + maturity_ratio * 155)
        color = f'#{0:02x}{green_value:02x}{0:02x}'

        self.canvas.create_oval(x - size, y - size, x + size, y + size,
                               fill=color, outline='', tags='plant')

    def _draw_animal(self, animal, show_energy, show_vision):
        """Draw an animal entity with enhanced visuals"""
        x = animal.x * self.scale
        y = animal.y * self.scale

        # Determine size based on species and genetics
        base_size = 3
        if isinstance(animal, Deer) or isinstance(animal, Wolf):
            base_size = 5
        size = base_size * animal.genetics['size']

        # Get color from entity
        color = animal.get_color()

        # Draw vision range if enabled
        if show_vision:
            vision_radius = animal.base_vision_range * animal.genetics['vision_range'] * self.scale
            self.canvas.create_oval(x - vision_radius, y - vision_radius,
                                   x + vision_radius, y + vision_radius,
                                   outline=color, width=1, dash=(2, 2), tags='vision')

        # Draw direction indicator if hunting or fleeing
        if animal.target:
            target_x = animal.target.x * self.scale
            target_y = animal.target.y * self.scale

            # Draw line to target
            line_color = '#ff0000' if animal.state == BehaviorState.HUNTING else '#00ff00'
            if animal.state in [BehaviorState.HUNTING, BehaviorState.FLEEING, BehaviorState.SEEKING_FOOD]:
                self.canvas.create_line(x, y, target_x, target_y,
                                       fill=line_color, width=1, dash=(3, 3), tags='target_line')

        # Draw animal body
        if isinstance(animal, (Rabbit, Deer)):
            # Herbivores as circles
            self.canvas.create_oval(x - size, y - size, x + size, y + size,
                                   fill=color, outline='white', width=1, tags='animal')
        else:
            # Carnivores as triangles (more aggressive look)
            points = [
                x, y - size * 1.3,  # top
                x - size, y + size,  # bottom left
                x + size, y + size   # bottom right
            ]
            self.canvas.create_polygon(points, fill=color, outline='white', width=1, tags='animal')

        # Draw energy bar if enabled
        if show_energy:
            bar_width = size * 3
            bar_height = 2
            bar_x = x - bar_width / 2
            bar_y = y - size - 5

            # Background (red)
            self.canvas.create_rectangle(bar_x, bar_y, bar_x + bar_width, bar_y + bar_height,
                                        fill='#ff0000', outline='', tags='energy_bg')

            # Foreground (green) based on energy level
            energy_ratio = max(0, min(1, animal.energy / animal.max_energy))
            energy_width = bar_width * energy_ratio

            # Color gradient: red -> yellow -> green
            if energy_ratio < 0.3:
                energy_color = '#ff0000'  # Red
            elif energy_ratio < 0.6:
                energy_color = '#ffaa00'  # Orange
            else:
                energy_color = '#00ff00'  # Green

            self.canvas.create_rectangle(bar_x, bar_y, bar_x + energy_width, bar_y + bar_height,
                                        fill=energy_color, outline='', tags='energy_fg')

        # Draw gender indicator (small dot)
        gender_color = '#4444ff' if animal.gender.value == 'male' else '#ff44ff'
        self.canvas.create_oval(x + size - 1, y + size - 1, x + size + 1, y + size + 1,
                               fill=gender_color, outline='', tags='gender')

    def update_statistics(self):
        """Update the statistics display"""
        stats = self.ecosystem.get_statistics()
        pops = stats['populations']

        # Build statistics text
        text = f"""
╔══════════════════════════════════════════╗
║          POPULATION OVERVIEW             ║
╚══════════════════════════════════════════╝

Time Step: {stats['time_step']}

🌱 Plants:           {pops['plants']:>6}
🐰 Rabbits:          {pops['rabbits']:>6}
🦌 Deer:             {pops['deer']:>6}
🦊 Foxes:            {pops['foxes']:>6}
🐺 Wolves:           {pops['wolves']:>6}

Total Herbivores:    {pops['total_herbivores']:>6}
Total Carnivores:    {pops['total_carnivores']:>6}

╔══════════════════════════════════════════╗
║          LIFETIME STATISTICS             ║
╚══════════════════════════════════════════╝

Total Births:        {stats['total_births']:>6}
Total Deaths:        {stats['total_deaths']:>6}

╔══════════════════════════════════════════╗
║          AVERAGE ATTRIBUTES              ║
╚══════════════════════════════════════════╝

Rabbits:
  Energy: {stats['rabbits_avg_energy']:>6.1f} / 80
  Age:    {stats['rabbits_avg_age']:>6.1f}

Deer:
  Energy: {stats['deer_avg_energy']:>6.1f} / 150
  Age:    {stats['deer_avg_age']:>6.1f}

Foxes:
  Energy: {stats['foxes_avg_energy']:>6.1f} / 100
  Age:    {stats['foxes_avg_age']:>6.1f}

Wolves:
  Energy: {stats['wolves_avg_energy']:>6.1f} / 180
  Age:    {stats['wolves_avg_age']:>6.1f}
"""

        self.stats_text.delete(1.0, tk.END)
        self.stats_text.insert(1.0, text)

    def update_graphs(self):
        """Update population graphs"""
        history = self.ecosystem.history

        # Only update every 10 steps to improve performance
        if self.ecosystem.time_step % 10 != 0:
            return

        self.plot.clear()

        # Plot each species
        if history['time']:
            self.plot.plot(history['time'], history['plants'], label='🌱 Plants',
                          color='#00aa00', linewidth=1.5, alpha=0.7)
            self.plot.plot(history['time'], history['rabbits'], label='🐰 Rabbits',
                          color='#228B22', linewidth=2)
            self.plot.plot(history['time'], history['deer'], label='🦌 Deer',
                          color='#8B4513', linewidth=2)
            self.plot.plot(history['time'], history['foxes'], label='🦊 Foxes',
                          color='#FF8C00', linewidth=2)
            self.plot.plot(history['time'], history['wolves'], label='🐺 Wolves',
                          color='#696969', linewidth=2)

        self.plot.set_xlabel('Time Step', fontsize=10)
        self.plot.set_ylabel('Population', fontsize=10)
        self.plot.set_title('Population Dynamics Over Time', fontsize=12, fontweight='bold')
        self.plot.legend(loc='upper right', fontsize=8, framealpha=0.9)
        self.plot.grid(True, alpha=0.3, linestyle='--')
        self.plot.set_facecolor('#f5f5f5')

        self.canvas_fig.draw()

    def update(self):
        """Main update loop"""
        if self.running:
            # Update simulation
            self.ecosystem.step()

            # Update visuals
            self.draw_entities()
            self.update_statistics()
            self.update_graphs()

        # Schedule next update
        self.master.after(self.speed, self.update)

    def toggle_pause(self):
        """Toggle simulation pause state"""
        self.running = not self.running
        if self.running:
            self.play_pause_btn.config(text="⏸ Pause")
        else:
            self.play_pause_btn.config(text="▶ Play")

    def change_speed(self, value):
        """Change simulation speed"""
        self.speed = int(value)

    def add_entity(self, entity_type):
        """Add a single entity at canvas click position"""
        # Add at random position for now
        self.ecosystem.add_entity(entity_type)

    def add_multiple_entities(self, entity_type, count):
        """Add multiple entities"""
        for _ in range(count):
            self.ecosystem.add_entity(entity_type)


def main():
    """Main entry point"""
    root = tk.Tk()
    root.title("🌍 Advanced Ecosystem Simulator - Predator-Prey Dynamics")
    root.geometry("1400x850")
    root.resizable(True, True)

    # Set icon and styling
    style = ttk.Style()
    style.theme_use('clam')

    app = SimulationGUI(root, world_size=100, canvas_size=700)

    root.mainloop()


if __name__ == "__main__":
    main()
