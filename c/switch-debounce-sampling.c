#include <stdint.h>

#define ISR __attribute__((signal, used, externally_visible))

static const uint8_t OUTPUT_MASK = 1 << 3;
static const uint8_t INPUT_MASK = 1 << 4;
static const uint8_t OCFA_MASK = 1 << 1;
static const uint8_t OCIEA_MASK = 1 << 1;
static const uint8_t PRESCALER_CLOCK_MASK = 0b00001011;
static const uint8_t COUNTER_OFF_MASK = 0b00001000;
static const uint8_t GLOBAL_INTERRUPT_ENABLE_MASK = 1 << 7;
static const uint8_t MATCHING_MAX_CNT = 20;
static volatile uint8_t* pinb = (uint8_t*)0x23;
static volatile uint8_t* ddrb = (uint8_t*)0x24;
static volatile uint8_t* portb = (uint8_t*)0x25;
static volatile uint8_t* pcmsk0 = (uint8_t*)0x6b;
static volatile uint8_t* timsk1 = (uint8_t*)0x6f;
static volatile uint8_t* sreg = (uint8_t*)0x5f;
static volatile uint8_t* pcicr = (uint8_t*)0x68;
static volatile uint8_t* tifr1 = (uint8_t*)0x36;

void counter_reset(uint16_t us);
void enable_timer();
void disable_timer();
void ISR __vector_3();
void ISR __vector_11();
void led_on() { *portb |= OUTPUT_MASK; }
void led_off() { *portb &= ~OUTPUT_MASK; }

void ISR __vector_3() // PCINT0
                      // 인터럽트
{
    enable_timer();
}

void enable_timer() { *timsk1 |= OCIEA_MASK; }
void disable_timer()
{
    *timsk1 &= ~OCIEA_MASK;
    *tifr1 = OCFA_MASK;
}

void interrupt_init(void)
{
    *pcmsk0 |= INPUT_MASK;

    *sreg |= GLOBAL_INTERRUPT_ENABLE_MASK;
    *pcicr |= 1;
}

void ISR __vector_11() // Timer1 Comp A 인터럽트
{
    static uint8_t matching_cnt = 0;
    static uint8_t input_before = 0;

    uint8_t input = *pinb & INPUT_MASK;

    if (input == input_before)
        matching_cnt += 1;
    else
    {
        input_before = input;
        matching_cnt = 0;
    }

    if (matching_cnt >= MATCHING_MAX_CNT)
    {
        if (input)
            led_on();
        else
            led_off();
        disable_timer();
        matching_cnt = 0;
    }
}

void counter_reset(uint16_t us)
{

    volatile uint8_t* ocr1ah = (uint8_t*)0x89;
    volatile uint8_t* ocr1al = (uint8_t*)0x88;
    volatile uint8_t* tcnt1l = (uint8_t*)0x84;
    volatile uint8_t* tcnt1h = (uint8_t*)0x85;
    volatile uint8_t* tccr1b = (uint8_t*)0x81;

    *tccr1b = COUNTER_OFF_MASK;

    *tcnt1h = 0;
    *tcnt1l = 0;

    uint8_t lsb = us / 4;
    uint8_t msb = (us / 4) >> 8;

    *ocr1ah = msb;
    *ocr1al = lsb;

    *tifr1 = OCFA_MASK;

    *tccr1b = PRESCALER_CLOCK_MASK;
}

int main(void)
{

    *ddrb |= OUTPUT_MASK;
    *ddrb &= ~INPUT_MASK;
    *portb &= ~OUTPUT_MASK;

    counter_reset(1000);
    interrupt_init();

    while (1)
    {
    }
}