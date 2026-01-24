import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

# 1. Load the data
df = pd.read_csv("output.csv")

# 2. Setup the figure and axis
fig, ax = plt.subplots(figsize=(8, 8))

# Get unique bodies and time steps
bodies = df['id'].unique()
times = df['time'].unique()

# 3. Create "actors" for the animation (one dot and one trail per body)
points = {}
trails = {}

for body_id in bodies:
    # 1. Plot the dot first to get a color from the cycle
    dot, = ax.plot([], [], 'o', label=f'Body {body_id}')
    
    # 2. Grab the color of that dot
    body_color = dot.get_color()
    
    # 3. Plot the trail line using the same color
    line, = ax.plot([], [], lw=1, alpha=0.5, color=body_color)

    points[body_id] = dot
    trails[body_id] = line

# Set plot limits based on the min/max coordinates in the file
padding = 2
ax.set_xlim(df['x'].min() - padding, df['x'].max() + padding)
ax.set_ylim(df['y'].min() - padding, df['y'].max() + padding)
ax.set_aspect('equal')
ax.grid(True)
ax.legend()

# 4. The Update Function (called for each frame)
def update(frame_time):
    # Filter data for the current time step
    current_step = df[df['time'] == frame_time]
    
    for body_id in bodies:
        body_data = current_step[current_step['id'] == body_id]
        
        if not body_data.empty:
            # Update the dot position
            points[body_id].set_data([body_data['x'].values[0]], [body_data['y'].values[0]])
            
            # Update the trail (optional: shows path up to current time)
            history = df[(df['id'] == body_id) & (df['time'] <= frame_time)]
            trails[body_id].set_data(history['x'], history['y'])
            
    return list(points.values()) + list(trails.values())

# 5. Create the animation
# interval=20 means 20ms between frames (~50fps)
ani = FuncAnimation(fig, update, frames=times, interval=1, blit=True)

plt.title("N-Body Simulation Animation")
plt.show()