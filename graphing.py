import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

# Example data lists
prey_population = []
predator_population = []
time_steps = []

def update(frame):
    global ecosystem  # Assume ecosystem is accessible
    prey_pop, pred_pop = ecosystem.get_populations()
    prey_population.append(prey_pop)
    predator_population.append(pred_pop)
    time_steps.append(frame)
    
    plt.cla()  # Clear the current axes
    plt.plot(time_steps, prey_population, label="Prey")
    plt.plot(time_steps, predator_population, label="Predators")
    plt.xlabel('Time Step')
    plt.ylabel('Population')
    plt.legend(loc="upper right")
    plt.tight_layout()

# Assuming your simulation step runs every 100ms, adjust interval accordingly
ani = FuncAnimation(plt.gcf(), update, interval=100)

plt.tight_layout()
plt.show()
