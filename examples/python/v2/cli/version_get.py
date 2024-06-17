import json
from platform_os from is_windows


if is_windows():
    from request import post
else:
    from sock import post


json_response = post("/internal/version/get")

print("The response is as follows.\n")
print(json_response)
