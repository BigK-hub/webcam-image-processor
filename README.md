# webcam-image-processor
This program, written in rust, applies image processing effects to your webcam feed.

## Currently these effects are implemented:
- Sobel
- Sobel with colour
- Threshold
- Threshold per channel
- Map brightness to colour palette
- Basic Kuwahara filter
- Random Bias Dithering
- Grid Pattern Dithering
- Floyd Steinberg Dithering
- Gaussian Blur 3x3
- Box Blur
- Emboss
- Outline
- Greyscale
- Chromatic Aberration
- Sharpen
- Sharpen with Colour
- CrossBlur (dreamed up kernel)

## How to use
- [S] key to snap a photo. 
- [H] to hide the UI.
- Use arrow keys to change effects. Up/Down to change input mode. Left/Right to change image processor. Alternatively you can use your mouse to drag the slider in the top left.

