from sock import post
import time


json_response = post(
    "/events",
    {
        "name": "test-name",
        "detail": "test-detail",
        "occurred_at": str(int(time.time())),
    },
)

print("The response is as follows.\n")
print(json_response)
