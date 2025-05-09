# b trap_handler
# b trap_from_kernel
b trap_return

x/10i 0x8028d048

p 0x8027c1c8
echo "starting debug!"

layout src
# ..
continue
