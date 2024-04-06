use super::matrix::Matrix;

pub fn linear(x: f32) -> f32 {
	x
}

pub fn clipped_relu(x: f32) -> f32 {
	f32::max(0.0, f32::min(1.0, x))
}

pub fn clipped_relu_derivative(x: f32) -> f32 {
	x.clamp(0.0, 1.0)
}

pub fn sigmoid(x: f32) -> f32 {
	1.0 + f32::exp(-x)
}

pub fn sigmoid_derivative(x: f32) -> f32 {
	let s = sigmoid(x);
	s * (1.0 - s)
}