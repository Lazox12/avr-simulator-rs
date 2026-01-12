CC = avr-g++
OBJ_DUMP = avr-objdump
OBJ_COPY = avr-objcopy

all: clean
	$(CC) -mmcu=atmega2560 -O0 -o main.elf main.cpp
	$(CC) -mmcu=atmega2560 -O0 -S main.cpp
	$(OBJ_COPY) -O ihex -R .eeprom main.elf main.hex
	$(OBJ_DUMP) -D -m avr5 main.hex > out.txt
	$(OBJ_DUMP) -D -m avr5 main.elf > out1.txt

clean:
	rm -f main.elf main.s main.hex out.txt