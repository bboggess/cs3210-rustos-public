#define GPIO_BASE (0x3F000000 + 0x200000)

volatile unsigned *GPIO_FSEL1 = (volatile unsigned *)(GPIO_BASE + 0x04);
volatile unsigned *GPIO_SET0  = (volatile unsigned *)(GPIO_BASE + 0x1C);
volatile unsigned *GPIO_CLR0  = (volatile unsigned *)(GPIO_BASE + 0x28);

static void spin_sleep_us(unsigned int us) {
  for (unsigned int i = 0; i < us * 6; i++) {
    asm volatile("nop");
  }
}

static void spin_sleep_ms(unsigned int ms) {
  spin_sleep_us(ms * 1000);
}

static void set_gpio_output_pin(unsigned pin) {
  *GPIO_FSEL1 = (1 << pin);
}

static void clear_gpio_pin(unsigned pin) {
  *GPIO_CLR0 = (1 << pin);
}

static void set_gpio_pin(unsigned pin) {
  *GPIO_SET0 = (1 << pin);
}

int kmain(void) {
  // FIXME: STEP 1: Set GPIO Pin 16 as output.
  set_gpio_output_pin(18);
  // FIXME: STEP 2: Continuously set and clear GPIO 16.
  int flag = 1;
  for (;;) {
    unsigned pin = 16;
    if (flag == 0) {
      clear_gpio_pin(pin);
    }
    else {
      set_gpio_pin(pin);
    }

    flag = (flag + 1) % 2;
    spin_sleep_ms(1000);
  }
}
