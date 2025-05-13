# Lottify - Proposal

Filip Ďuriš

## Introduction

Lottify is a tool for generating optimized SVGs and [Lottie files](<https://en.wikipedia.org/wiki/Lottie_(file_format)>) from glTF models. Lottify aims to decrease the difficulty of creating Lotties animations. The currently available tooling is very basic, the most popular being [LottieLab](https://www.lottielab.com/dashboard) and [LottieFiles](https://lottiefiles.com), neither providing any advanced tooling and mostly just rely on simple vector graphics manipulation and interpolation. Adobe recently announced their project called [Turntable](https://www.adobe.com/max/2024/sessions/project-turntable-gs3-9.html) which utilizes AI to aid in the creation and animation of SVGs, but it will likely end up being closed-source and very expensive given Adobe's track record. Lottify therefore takes a different avenue, which is taking advantage the many 3D modeling tools for the creation of the animation and then simply generating an SVG or Lottie from said model.

## Requirments

- Vectorize arbitrary 3D model or collection of models using an edge-finding algorithm
- Turn a single vectorized frame into a still SVG or a collection of vectorized frames into an animated Lottie file
- Transform the model in 3D space before vectorization to simulate camera position and orientation
- Colour the vectorized shapes based on the colour of the model's shaders (texture support might also be a consideration)
- Re-order the vectorized shapes before exporting to simulate 3D occlusion
- Preview and edit still frames and animations in a GUI mode before exporting

### Secondary requirements

- Interpolate between frames of a Lottie animation instead of just using collection of still frames
- Automatic shape ordering computed based on 3D occlusion
- Running the algorithm on the GPU for real-time preview
- Rotate camera in real time

## Dependencies

- bevy - for rendering the GUI and working with the 3D models
- bevy_egui - simple GUI library for bevy
- bevy_vello - support for rendering vector graphics in bevy
- serde & serde_json - for serializing Lottie files
- esvg - for serialzing SVGs

![3d](images/penguin_3dmodel.png)
Fig. 1: 3D Model

![svg](images/penguin_vector.svg)
Fig. 2: Vectorized
