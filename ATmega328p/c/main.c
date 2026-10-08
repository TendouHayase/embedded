void loop(void);

int main(void)
{
    volatile unsigned char* ddrb = (unsigned char*)0x24;
    volatile unsigned char* portb = (unsigned char*)0x25;
    unsigned char MASK = 1 << 5;
    *ddrb |= MASK;
    while (1)
    {
        loop();
        *portb ^= MASK;
    }
}

void loop(void)
{
    for (volatile unsigned int i = 0; i < 100; ++i)
    {
        for (volatile unsigned int j = 0; j < 10000; ++j)
        {
        }
    }
}