from reachy_mini import ReachyMini
from reachy_mini.utils import create_head_pose
import numpy as np

with ReachyMini(connection_mode="localhost_only") as mini:
    # Move everything at once
    for _ in range(100):
        mini.goto_target(
            head=create_head_pose(z=10, mm=True),    # Up 10mm
            antennas=np.deg2rad([45, 45]),           # Antennas out
            body_yaw=np.deg2rad(0),                 # Turn body
            duration=2.0,                            # Take 2 seconds
            method="minjerk"                         # Smooth acceleration
        )
        mini.goto_target(
            head=create_head_pose(z=10, mm=True),    # Up 10mm
            antennas=np.deg2rad([-45, -45]),           # Antennas out
            duration=2.0,                            # Take 2 seconds
            method="minjerk"                         # Smooth acceleration
        )