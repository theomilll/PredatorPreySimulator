"""
Alternative graphing module for standalone population visualization
Can be used independently or integrated with other analysis tools
"""

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation


class PopulationGrapher:
    """Standalone grapher for ecosystem population data"""

    def __init__(self, ecosystem):
        self.ecosystem = ecosystem
        self.fig, self.ax = plt.subplots(figsize=(10, 6))
        self.lines = {}

    def setup_plot(self):
        """Configure the plot appearance"""
        self.ax.set_xlabel('Time Step', fontsize=12)
        self.ax.set_ylabel('Population', fontsize=12)
        self.ax.set_title('Ecosystem Population Dynamics', fontsize=14, fontweight='bold')
        self.ax.grid(True, alpha=0.3, linestyle='--')
        self.ax.set_facecolor('#f5f5f5')

    def update_plot(self, frame):
        """Update function for animation"""
        # Run simulation step
        self.ecosystem.step()

        history = self.ecosystem.history

        # Clear and redraw
        self.ax.clear()
        self.setup_plot()

        if history['time']:
            # Plot all species
            self.ax.plot(history['time'], history['plants'], label='🌱 Plants',
                        color='#00aa00', linewidth=1.5, alpha=0.7)
            self.ax.plot(history['time'], history['rabbits'], label='🐰 Rabbits',
                        color='#228B22', linewidth=2)
            self.ax.plot(history['time'], history['deer'], label='🦌 Deer',
                        color='#8B4513', linewidth=2)
            self.ax.plot(history['time'], history['foxes'], label='🦊 Foxes',
                        color='#FF8C00', linewidth=2)
            self.ax.plot(history['time'], history['wolves'], label='🐺 Wolves',
                        color='#696969', linewidth=2)

            self.ax.legend(loc='upper right', fontsize=10, framealpha=0.9)

    def animate(self, interval=50, frames=1000):
        """Create animated plot"""
        self.setup_plot()
        anim = FuncAnimation(self.fig, self.update_plot, frames=frames,
                           interval=interval, repeat=False)
        plt.show()

    def plot_static(self):
        """Create static plot from current history"""
        history = self.ecosystem.history

        self.setup_plot()

        if history['time']:
            self.ax.plot(history['time'], history['plants'], label='🌱 Plants',
                        color='#00aa00', linewidth=1.5, alpha=0.7)
            self.ax.plot(history['time'], history['rabbits'], label='🐰 Rabbits',
                        color='#228B22', linewidth=2)
            self.ax.plot(history['time'], history['deer'], label='🦌 Deer',
                        color='#8B4513', linewidth=2)
            self.ax.plot(history['time'], history['foxes'], label='🦊 Foxes',
                        color='#FF8C00', linewidth=2)
            self.ax.plot(history['time'], history['wolves'], label='🐺 Wolves',
                        color='#696969', linewidth=2)

            self.ax.legend(loc='upper right', fontsize=10, framealpha=0.9)

        plt.tight_layout()
        plt.show()


def create_comparison_plots(ecosystems_dict):
    """
    Create comparison plots for multiple ecosystem runs

    Args:
        ecosystems_dict: Dictionary of {label: ecosystem} pairs
    """
    fig, axes = plt.subplots(2, 3, figsize=(15, 10))
    fig.suptitle('Multi-Ecosystem Comparison', fontsize=16, fontweight='bold')

    species = ['plants', 'rabbits', 'deer', 'foxes', 'wolves']
    colors = ['#00aa00', '#228B22', '#8B4513', '#FF8C00', '#696969']

    ax_idx = 0
    for species_name, color in zip(species, colors):
        ax = axes[ax_idx // 3, ax_idx % 3]

        for label, ecosystem in ecosystems_dict.items():
            history = ecosystem.history
            if history['time']:
                ax.plot(history['time'], history[species_name],
                       label=label, linewidth=2, alpha=0.8)

        ax.set_title(f'{species_name.capitalize()} Population', fontweight='bold')
        ax.set_xlabel('Time Step')
        ax.set_ylabel('Population')
        ax.legend()
        ax.grid(True, alpha=0.3)

        ax_idx += 1

    # Use last subplot for total populations
    ax = axes[1, 2]
    for label, ecosystem in ecosystems_dict.items():
        history = ecosystem.history
        if history['time']:
            total_animals = [r + d + f + w for r, d, f, w in
                           zip(history['rabbits'], history['deer'],
                               history['foxes'], history['wolves'])]
            ax.plot(history['time'], total_animals, label=label, linewidth=2)

    ax.set_title('Total Animal Population', fontweight='bold')
    ax.set_xlabel('Time Step')
    ax.set_ylabel('Population')
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.show()


# Example usage
if __name__ == "__main__":
    from ecosystem import Ecosystem

    # Create ecosystem
    eco = Ecosystem(100, 100)

    # Create grapher and animate
    grapher = PopulationGrapher(eco)
    grapher.animate(interval=50, frames=500)
