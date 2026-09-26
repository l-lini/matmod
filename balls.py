import pygame

# Code based on Java code written by Simon Robillard
# The code has intentionally been kept as simple as possible, but if you wish, you can improve the design.

# Constants 

## Model constants
WIDTH = 4 # width of the bounding box in meters
HEIGHT = 3 # height of the bounding box in meters

## Display constants
PIXELS_PER_METER = 200 # This many pixels on the screen represent one meter in the model
SCREEN_SIZE = WIDTH * PIXELS_PER_METER, HEIGHT * PIXELS_PER_METER

## Colors
WHITE = 255, 255, 255
RED = 255, 0, 0

## Speed of the simulation
## One second real time equals one second in the simulation
FRAMES_PER_SECOND = 60

GRAVITY = 9.82

# A simple record class describing a ball
class Ball:
    def __init__(self, x, y, vx, vy, r):
        self.x = x
        self.y = y
        self.vx = vx
        self.vy = vy
        self.r = r
        self.m = r ** 3

    def time_travel(self, t):
        self.vy -= GRAVITY * t / 2
        self.x += self.vx * t
        self.y += self.vy * t
        self.vy -= GRAVITY * t / 2

def dot(x1, y1, x2, y2) -> float:
    return x1 * x2 + y1 * y2

def normalize(x, y) -> (float, float):
    return (x / length(x,y), y / length(x,y))

def length(x, y) -> float:
    return length_squared(x, y) ** 0.5

def distance(x1, y1, x2, y2) -> float:
    return length(x2 - x1, y2 - y1)

def length_squared(x, y) -> float:
    return x**2 + y**2

def distance_squared(x1, y1, x2, y2) -> float:
    return length_squared(x2 - x1, y2 - y1)

def one_dimensional_collision(v1, v2, m1, m2) -> (float, float):
    R = v2 - v1
    I = v1 * m1 + v2 * m2
    new_v1 = (I + R) / (1 + m1 / m2)
    new_v2 = new_v1 - R
    return (new_v1, new_v2)

def two_dimensional_collision(b1, b2):
    # Normalized collision vector
    cvx, cvy  = normalize(b2.x - b1.x, b2.y - b1.y)

    # Calculate paralell velocities
    v1 = dot(b1.vx, b1.vy, cvx, cvy)
    v2 = dot(b2.vx, b2.vy, cvx, cvy)

    # Calculate new paralell velocities
    (new_v1, new_v2) = one_dimensional_collision(v1, v2, b1.m, b2.m)
    # Convert new velocity into original coordinate system
    new_v1x = cvx * new_v1
    new_v1y = cvy * new_v1
    new_v2x = cvx * new_v2
    new_v2y = cvy * new_v2

    # Invert paralell velocity
    b1.vx += new_v1x * 2
    b1.vy += new_v1y * 2
    b2.vx += new_v2x * 2
    b2.vy += new_v2y * 2

def time_until_wall(p, v, r) -> float:
    t = (r - p) / v
    return t if t > 0 else None

class SignedWallDistance:
    def __init__(self, p, v, r):
        self.a = v
        self.b = p - r

    def __call__(self, t):
        return self.a * t + self.b

    def collision_time(self):
        if self.a == 0:
            return 0 if abs(self.b) < 1e-3 else None
        t = - self.b / self.a
        return t if self.slope() < 0 else None

    def slope(self):
        return self.a

class SignedSquaredBallDistance:
    def __init__(self, b1, b2):
        r = b1.r + b2.r
        dvx = b2.vx - b1.vx
        dvy = b2.vy - b1.vy
        dx = b2.x - b1.x
        dy = b2.y - b1.y

        self.a = dvx ** 2 + dvy ** 2
        self.b = 2 * dx * dvx + 2 * dy * dvy
        self.c = dx ** 2 + dy ** 2 - r ** 2

    def roots(self) -> [float]:
        radicand = self.b ** 2 - 4 * self.a * self.c

        if radicand < 0:
            return []
        
        t1 = (- self.b - radicand ** 0.5) / (2 * self.a)
        t2 = (- self.b + radicand ** 0.5) / (2 * self.a)

        return [t1, t2]

    def collision_time(self) -> float:
        for root in self.roots():
            if self.slope(root) < 0:
                return root
        return None

    def __call__(self, t) -> float:
        return self.a * t ** 2 + self.b * t + self.c

    def slope(self, t) -> float:
        return 2 * self.a * t + self.b

# The mathematical model
class Model:
    def __init__(self):
        # Initialize the model with a few balls
        self.balls = [Ball(1, 2.5, 1.2, 1.6, 0.2),
                      Ball(2, 1.5, -0.6, 0.6, 0.3),
                      Ball(3, 0.5, -0.5, 0.6, 0.5),
                      ]

    def next(self, deltaT):
        # This method implements one step of the simulation with a time interval of deltaT seconds
        for i in range(len(self.balls)):
            dt = deltaT
            b = self.balls[i]
            for j in range(i + 1, len(self.balls)):
                b2 = self.balls[j]

                signed_squared_distance_function = SignedSquaredBallDistance(b, b2)
                collision_time = signed_squared_distance_function.collision_time()

                if collision_time and collision_time > 0 and collision_time < dt:
                    b.time_travel(collision_time)
                    # b2.time_travel(collision_time)
                    dt -= collision_time

                    two_dimensional_collision(b, b2)

            t_right = SignedWallDistance(WIDTH - b.x, - b.vx, b.r).collision_time()
            if t_right and t_right < dt:
                b.time_travel(t_right)
                b.vx = - abs(b.vx)
                dt -= t_right
            t_top = SignedWallDistance(HEIGHT - b.y, - b.vy, b.r).collision_time()
            if t_top and t_top < dt:
                b.time_travel(t_top)
                b.vy = - abs(b.vy)
                dt -= t_top
            t_left = SignedWallDistance(b.x, b.vx, b.r).collision_time()
            if t_left and t_left < dt:
                b.time_travel(t_left)
                b.vx = abs(b.vx)
                dt -= t_left
            t_bottom = SignedWallDistance(b.y, b.vy, b.r).collision_time()
            if t_bottom and t_bottom < dt:
                b.time_travel(t_bottom)
                b.y = b.r
                b.vy = abs(b.vy)
                dt -= t_bottom

            b.time_travel(dt)
                    
# Initialization
pygame.init()
clock = pygame.time.Clock()
screen = pygame.display.set_mode(SCREEN_SIZE)
model = Model()

# Main loop
running = True
while running:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False

    # Update the model
    model.next(1 / FRAMES_PER_SECOND)

    # Draw the state of the model on the screen
    screen.fill(WHITE)
    for b in model.balls:
        pygame.draw.circle(screen, RED,
                           (b.x * PIXELS_PER_METER,
                            (HEIGHT - b.y) * PIXELS_PER_METER), # In the model, we have y = 0 at the bottom of the bounding box and y = HEIGHT at the top
                                                                # Pygame's draw method wants y = 0 at the top of the display window, increasing to its max value at the bottom
                           b.r * PIXELS_PER_METER)
    pygame.display.flip()

    # Wait then loop back
    clock.tick(FRAMES_PER_SECOND)
