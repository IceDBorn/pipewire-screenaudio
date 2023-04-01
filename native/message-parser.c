#include <stdio.h>
#include <stdlib.h>

int main (int argc, char *argv[]) {
  size_t INT_SIZE = sizeof(int);
  unsigned char* buffer;

  buffer = (unsigned char*)malloc(INT_SIZE);
  for (int i = 0; i < INT_SIZE; i++)
    buffer[i] = getchar();

  /*
  for (int i = 0; i < INT_SIZE / 2; i++) {
    unsigned char temp = buffer[i];
    buffer[i] = buffer[INT_SIZE - 1 - i];
    buffer[INT_SIZE - 1 - i] = temp;
  }
  */

  int charactersToRead = *(int*)buffer;
  // printf("charactersToRead: %d", charactersToRead);
  free(buffer);

  buffer = (unsigned char*)malloc(charactersToRead + 1);
  buffer[charactersToRead] = '\0';
  for (int i = 0; i < charactersToRead; i++)
    buffer[i] = getchar();

  printf("%s", buffer);
  free(buffer);

  return 0;
}

