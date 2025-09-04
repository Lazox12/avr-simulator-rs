CC = avr-g++
OBJ_DUMP = avr-objdump
OBJ_COPY = avr-objcopy

all: clean
	$(CC) -mmcu=atxmega128b3 -o main.elf main.s
	$(OBJ_COPY) -O ihex -R .eeprom main.elf main.hex
	$(OBJ_DUMP) -D -m avr5 main.hex > out.txt

clean:
	rm -f main.elf main.hex out.txt