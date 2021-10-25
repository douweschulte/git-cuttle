pub struct Area {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

impl Area {
    pub fn new(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        Area {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }
    pub fn center(&self) -> Point {
        Point(
            (self.end_x - self.start_x) / 2.0 + self.start_x,
            (self.end_y - self.start_y) / 2.0 + self.start_y,
        )
    }
    pub fn shrink(&self, factor: f64) -> Self {
        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        Area {
            start_x: self.start_x + dx * factor,
            end_x: self.end_x - dx * factor,
            start_y: self.start_y + dy * factor,
            end_y: self.end_y - dy * factor,
        }
    }
    /// Split the area at the given point (in fractions: 0.5 is halfway)
    pub fn split_horizontally(&self, point: f64) -> (Self, Self) {
        let dx = self.end_x - self.start_x;
        (
            Area {
                start_x: self.start_x,
                end_x: self.start_x + dx * point,
                start_y: self.start_y,
                end_y: self.end_y,
            },
            Area {
                start_x: self.start_x + dx * point,
                end_x: self.end_x,
                start_y: self.start_y,
                end_y: self.end_y,
            },
        )
    }
    /// Split the area at the given point (in fractions: 0.5 is halfway)
    pub fn split_vertically(&self, point: f64) -> (Self, Self) {
        let dy = self.end_y - self.start_y;
        (
            Area {
                start_x: self.start_x,
                end_x: self.end_x,
                start_y: self.start_y,
                end_y: self.start_y + dy * point,
            },
            Area {
                start_x: self.start_x,
                end_x: self.end_x,
                start_y: self.start_y + dy * point,
                end_y: self.end_y,
            },
        )
    }
    pub fn split_evenly(&self, size: (usize, usize)) -> Vec<Self> {
        let step_x = (self.end_x - self.start_x) / (size.0 as f64);
        let step_y = (self.end_y - self.start_y) / (size.1 as f64);
        let mut output = Vec::with_capacity(size.0 * size.1);
        for x in 0..size.0 {
            for y in 0..size.1 {
                output.push(Area {
                    start_x: self.start_x + step_x * (x as f64),
                    end_x: self.start_x + step_x * ((x + 1) as f64),
                    start_y: self.start_y + step_y * (y as f64),
                    end_y: self.start_y + step_y * ((y + 1) as f64),
                })
            }
        }
        output
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point(pub f64, pub f64);

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self(self.0 * other, self.1 * other)
    }
}

impl Point {
    pub fn normalize(&self) -> Self {
        let sum = self.0.abs() + self.1.abs();
        if sum == 0.0 {
            Point(0.0, 0.0)
        } else {
            Point(self.0 / sum, self.1 / sum)
        }
    }

    pub fn distance(&self, other: Self) -> f64 {
        ((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)).sqrt()
    }
}
