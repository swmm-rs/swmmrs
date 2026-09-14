# LLDB summary formatter for SWMM DateTime values.
import datetime

import lldb

BASE = datetime.datetime(1899, 12, 30)


def datetime_summary(valobj, _internal_dict):
    try:
        child = valobj.GetChildAtIndex(0)
        data = child.GetData()
        err = lldb.SBError()  # pyright: ignore[reportAttributeAccessIssue]
        days = data.GetDouble(err, 0)
        if err.Fail():
            return f"DateTime(<unavailable>: {err.GetCString()})"
        dt = BASE + datetime.timedelta(days=days)
        return f"{days:.6f} days -> {dt.isoformat()}Z"
    except Exception as e:
        return f"DateTime(<error: {e}>)"


def __lldb_init_module(debugger, _):
    regex = r".*::DateTime$"
    debugger.HandleCommand(
        f'type summary add -x "{regex}" -F {__name__}.datetime_summary'
    )
    debugger.HandleCommand("type category enable default")
