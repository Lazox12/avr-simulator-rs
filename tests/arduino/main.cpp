#include <avr/io.h>

int main(){
    while (true){
        PORTD=1;
        PORTD=0;
    }
}