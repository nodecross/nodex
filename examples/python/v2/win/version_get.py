from request import post


json_response = post("/internal/version/get")

print("The response is as follows.\n")
print(json_response)
