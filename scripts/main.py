import sys
from controller import Controller


def main():
    controller = Controller()
    sys.exit(controller.run())

if __name__ == '__main__':
    main()