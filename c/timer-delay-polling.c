#include <stdint.h>

static const uint32_t CPU_HZ = 16000000UL;
static const uint8_t OUTPUT_MASK = 1 << 3;
static const uint8_t INPUT_MASK = 1 << 4;
static const uint8_t OCFA_MASK = 1 << 1;
static const uint16_t PRESCALER_CLOCK = 1024;
static const uint8_t PRESCALER_CLOCK_MASK = 0b00001101;
static const uint8_t COUNTER_OFF_MASK = 0b00001000;

void counter_polling(uint16_t value)
{

    volatile uint8_t* ocr1ah = (uint8_t*)0x89;
    volatile uint8_t* ocr1al = (uint8_t*)0x88;
    volatile uint8_t* tcnt1l = (uint8_t*)0x84;
    volatile uint8_t* tcnt1h = (uint8_t*)0x85;
    volatile uint8_t* tccr1b = (uint8_t*)0x81;
    volatile uint8_t* tifr1 = (uint8_t*)0x36;

    *tccr1b = COUNTER_OFF_MASK;

    *tcnt1h = 0;
    *tcnt1l = 0;

    uint8_t lsb = value;
    uint8_t msb = value >> 8;

    *ocr1ah = msb;
    *ocr1al = lsb;

    *tifr1 = OCFA_MASK; // 1을 쓸경우 플래그가 지워지고 0을 쓰면 본래 그대로.
                        // 하드웨어 규칙이라는데 왜 이런 규칙을 만들었을까요?

    *tccr1b = PRESCALER_CLOCK_MASK;

    while (!(*tifr1 & OCFA_MASK))
    {
    }
}

int main(void)
{
    volatile uint8_t* pinb = (uint8_t*)0x23;
    volatile uint8_t* ddrb = (uint8_t*)0x24;
    volatile uint8_t* portb = (uint8_t*)0x25;

    *ddrb |= OUTPUT_MASK;
    *ddrb &= ~INPUT_MASK;
    *portb &= ~OUTPUT_MASK;

    while (1)
    {
        if (*pinb & INPUT_MASK)
            *portb |= OUTPUT_MASK;
        else
            *portb &= ~OUTPUT_MASK;
        counter_polling(CPU_HZ / PRESCALER_CLOCK);
    }
}