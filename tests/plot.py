import matplotlib.pyplot as plt
import numpy as np

# read binary file
data = np.fromfile('../output/output.png', dtype=np.uint8)

print(data.shape)

# plot data
plt.plot(data)
plt.show()
plt.savefig('../output/plot.png')  # Saves the plot as a PNG file

