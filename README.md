Reproduces a segfault that occurs when using nvidia drivers on Wayland. When opening multiple winit windows, then closing one, a segfault occurs because libnvidia incorrectly frees some resource that all windows depend on.

Example bad driver versions: 570.144, 575.51.02, and 580.126.18 through 580.159.03

See the first bug report for this issue here: https://forums.developer.nvidia.com/t/crash-on-wayland-wsi-functions-with-multiple-vulkan-instances-570-144-575-51-02/331981


Repro steps:
1. Compile and run this program (install cargo, then type "cargo run").
2. Open a second window using "Debug Window" >> "Status".
3. Close either window.
4. Observe the segfault
