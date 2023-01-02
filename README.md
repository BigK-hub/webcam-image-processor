# webcam-image-processor
This is a rust based program, which activates the webcam and applies filters to it.
Currently there are implemented

- Sobel
- Sobel with colour
- Threshold
- Gaussian Blur 3x3
- Box Blur 

these mode are currently able to be set via a variable called "mode" in the main.rs file

```rust
let mode = Mode::Sobel;
```

using the s key there will be a screenshot taken of the current frame and saved in the proect folder. 

