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

# GRAVITY = 0.982
GRAVITY = 0.0

# A simple record class describing a ball
class Ball:
    def __init__(self, x, y, vx, vy, radius):
        self.x = x
        self.y = y
        self.vx = vx
        self.vy = vy
        self.radius = radius

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

def two_dimensional_collision(p1x, p1y, v1x, v1y, p2x, p2y, v2x, v2y, m1, m2) -> ((float, float), (float, float)):
    # Normalized collision vector
    cvx, cvy  = normalize(p2x - p1x, p2y - p1y)

    # Calculate paralell velocities
    v1 = dot(v1x, v1y, cvx, cvy)
    v2 = dot(v2x, v2y, cvx, cvy)

    # Calculate new paralell velocities
    (new_v1, new_v2) = one_dimensional_collision(v1, v2, m1, m2)
    # Convert new velocity into original coordinate system
    new_v1x = cvx * new_v1
    new_v1y = cvy * new_v1
    new_v2x = cvx * new_v2
    new_v2y = cvy * new_v2

    # Invert paralell velocity
    v1x += new_v1x * 2
    v1y += new_v1y * 2
    v2x += new_v2x * 2
    v2y += new_v2y * 2

    return ((v1x, v1y), (v2x, v2y))

def time_until_wall(p, v, r) -> float:
    return (r - p) / v

def quadratic_derivative(a, b, t) -> float:
    return 2 * a * t + b

def time_until_ball(x1, y1, v1x, v1y, x2, y2, v2x, v2y, r1, r2) -> float:
    r = r1 + r2
    dvx = v2x - v1x
    dvy = v2y - v1y
    dx = x2 - x1
    dy = y2 - y1

    a = dvx ** 2 + dvy ** 2
    b = 2 * dx * dvx + 2 * dy * dvy
    c = dx ** 2 + dy ** 2 - r ** 2

    radicand = b ** 2 - 4 * a * c

    if radicand <= 0:
        return None

    t1 = (- b - radicand ** 0.5) / (2 * a)
    t2 = (- b + radicand ** 0.5) / (2 * a)

    if quadratic_derivative(a, b, t1) < 0:
        return t1

    if quadratic_derivative(a, b, t2) < 0:
        return t2

    return None

# The mathematical model
class Model:
    def __init__(self):
        # Initialize the model with a few balls
        self.balls = [Ball(1, 2.5, 1.2, 1.6, 0.2),
                      Ball(2, 1.5, -0.6, 0.6, 0.3)]

    def next(self, deltaT):
        # This method implements one step of the simulation with a time interval of deltaT seconds

        for b in self.balls:
            # Detect collision with other balls
            for b2 in self.balls:
                t = time_until_ball(b.x, b.y, b.vx, b.vy, b2.x, b2.y, b2.vx, b2.vy, b.radius, b2.radius)
                if b != b2 and t and t < deltaT:
                    print(t)
                    b.x += b.vx * t
                    b.y += b.vy * t
                    b2.x += b2.vx * t
                    b2.y += b2.vy * t

                    m1 = b.radius ** 2
                    m2 = b2.radius ** 2
                    ((new_bvx, new_bvy), (new_b2vx, new_b2vy)) = two_dimensional_collision(b.x, b.y, b.vx, b.vy, b2.x, b2.y, b2.vx, b2.vy, m1, m2)
                    b.vx = new_bvx
                    b.vy = new_bvy
                    b2.vx = new_b2vx
                    b2.vy = new_b2vy
                    b.vx = 0
                    b.vy = 0
                    b2.vx = 0
                    b2.vy = 0

            # Detect collision with border
            if (b.x < b.radius or b.x > WIDTH - b.radius):
                b.vx = -b.vx # flip velocity horizontally

            if (b.y < b.radius or b.y > HEIGHT - b.radius):
                b.vy = -b.vy # flip velocity vertically


            # update position of ball based on velocity
            b.x = b.x + b.vx * deltaT
            b.y = b.y + b.vy * deltaT

            b.vy -= GRAVITY

                    
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
                           b.radius * PIXELS_PER_METER)
    pygame.display.flip()

    # Wait then loop back
    clock.tick(FRAMES_PER_SECOND)
