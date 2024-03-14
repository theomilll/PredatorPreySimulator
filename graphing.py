import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

prey_population = []
predator_population = []
time_steps = []

def update(frame):
    global ecosystem
    prey_pop, pred_pop = ecosystem.get_populations()
    prey_population.append(prey_pop)
    predator_population.append(pred_pop)
    time_steps.append(frame)
    
    plt.cla()
    plt.plot(time_steps, prey_population, label="Prey")
    plt.plot(time_steps, predator_population, label="Predators")
    plt.xlabel('Time Step')
    plt.ylabel('Population')
    plt.legend(loc="upper right")
    plt.tight_layout()

ani = FuncAnimation(plt.gcf(), update, interval=100)

plt.tight_layout()
plt.show()
