int f(int size)
{
    int *a = (int *)malloc(sizeof(int) * size);
    *(a + 1024) = 1;
}