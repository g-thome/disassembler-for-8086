printf "bits 16\n\n\n$1" > temp.asm;
nasm temp.asm -o "temp_bin";
xxd -b temp_bin;
rm temp_bin temp.asm;