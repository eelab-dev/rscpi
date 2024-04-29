# Function to generate a linear ramp
def generate_linear_ramp(start, end, n_samples):
    step = (end - start) / (n_samples - 1)
    return [int(round(start + step * i)) for i in range(n_samples)]

# Define the start, end, and number of samples
start_value = -32767
end_value = 32767
N = 100000  # Replace 100 with your desired number of samples

# Generate the ramp
ramp_samples = generate_linear_ramp(start_value, end_value, N)

# Save the samples to a file
file_path = 'linear_ramp.txt'
with open(file_path, 'w') as file:
    for sample in ramp_samples:
        file.write(f"{sample}\n")

print(f"Linear ramp with {N} samples saved to {file_path}")
