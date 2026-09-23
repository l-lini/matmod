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

GRAVITY = 0.982

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
    return (x**2 + y**2) ** 0.5

def distance(x1, y1, x2, y2) -> float:
    return length(x2 - x1, y2 - y1)

# The mathematical model
class Model:
    def __init__(self):
        # Initialize the model with a few balls
        self.balls = [Ball(1, 2.5, 1.2, 1.6, 0.2),
                      Ball(2, 1.5, -0.6, 0.6, 0.3)]

    def next(self, deltaT):
        # This method implements one step of the simulation with a time interval of deltaT seconds

        for b in self.balls:
            bx = b.x + b.vx * deltaT
            by = b.y + b.vy * deltaT

            # Detect collision with other balls
            for b2 in self.balls:
                if b != b2 and distance(bx, by, b2.x, b2.y) < b.radius + b2.radius:
                    # Normalized collision vector
                    cvx, cvy  = normalize(b2.x - bx, b2.y - by)

                    # Calculate paralell velocities
                    v1 = dot(b.vx, b.vy, cvx, cvy)
                    v2 = dot(b2.vx, b2.vy, cvx, cvy)

                    m1 = b.radius ** 3
                    m2 = b2.radius ** 3

                    # Calculate new paralell velocities
                    R = v2 - v1
                    I = v1 * m1 + v2 * m2
                    v1_ = (I + R) / (1 + m1 / m2)
                    v2_ = v1_ - R

                    # Convert new velocity into original coordinate system
                    v1_x = cvx * v1_
                    v1_y = cvy * v1_
                    v2_x = cvx * v2_
                    v2_y = cvy * v2_

                    # Invert paralell velocity
                    b.vx += v1_x * 2
                    b.vy += v1_y * 2
                    b2.vx += v2_x * 2
                    b2.vy += v2_y * 2
            # Future position
            bx = b.x + b.vx * deltaT
            by = b.y + b.vy * deltaT

            # Detect collision with border
            if (bx < b.radius or bx > WIDTH - b.radius):
                b.vx = -b.vx # flip velocity horizontally

            if (by < b.radius or by > HEIGHT - b.radius):
                b.vy = - 0.9 * b.vy # flip velocity vertically


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
