Reproduces a segfault that occurs when using old nvidia driver versions on Wayland. When opening multiple winit windows, then closing one, a segfault occurs because libnvidia incorrectly frees a resource that all windows depend on.

Upgrading from 580.126.18 to 580.159.03 fixes this.

Other example bad versions:
570.144, 575.51.02

See the first bug report for this issue here: https://forums.developer.nvidia.com/t/crash-on-wayland-wsi-functions-with-multiple-vulkan-instances-570-144-575-51-02/331981


Repro steps:
1. Compile and run this program (install cargo, then type "cargo run").
2. Open a second window using "Debug Window" >> "Status".
3. Close either window.
4. Observe the segfault
