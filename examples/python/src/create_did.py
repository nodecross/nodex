from sock import post


json_response = post("/identifiers")

print("The response is as follows.\n")
print(json_response)
