from platform_os import is_windows


if is_windows():
    from request import get
else:
    from sock import get

# PLEASE WRITE device_name
device_name = "DEVICE_NAME"

json_response = get(f"/identifiers?device_name={device_name}")

print("The response is as follows.\n")
print(json_response)
