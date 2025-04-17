import gdb
from gdb import printing

class DebugPrinter:
    """自动打印所有字段的 Pretty Printer"""

    def __init__(self, val):
        self.val = val  # gdb.Value

    def to_string(self):
        if self.val.type.code == gdb.TYPE_CODE_PTR:
            if self.val == 0:
                return "null"
            return self.val.dereference()

        # 如果是结构体/枚举，递归打印字段
        if self.val.type.code == gdb.TYPE_CODE_STRUCT:
            return self._format_struct()
        elif self.val.type.code == gdb.TYPE_CODE_UNION:
            return self._format_union()
        elif self.val.type.code == gdb.TYPE_CODE_ARRAY:
            return self._format_array()
        else:
            return str(self.val)

    def _format_struct(self):
        """格式化结构体"""
        fields = []
        for field in self.val.type.fields():
            field_name = field.name
            field_val = self.val[field_name]
            fields.append(f"{field_name}={DebugPrinter(field_val).to_string()}")
        return f"{self.val.type.name} {{ " + ", ".join(fields) + " }}"

    def _format_union(self):
        """格式化联合体"""
        return f"{self.val.type.name} {{ ... }}"

    def _format_array(self):
        """格式化数组"""
        elements = []
        for i in range(self.val.type.range()[1]):
            elements.append(DebugPrinter(self.val[i]).to_string())
        return f"[{', '.join(elements)}]"

# 注册全局 Pretty Printer
def register_printers():
    printing.register_pretty_printer(
        None,
        {
            "TaskControlBlock": lambda val: DebugPrinter(val).to_string(),
            "TaskUserResource": lambda val: DebugPrinter(val).to_string(),
        }
    )

register_printers()