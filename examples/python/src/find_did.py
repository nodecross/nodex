from sock import get


# PLEASE WRITE destination_did
destination_did = (
    "did:nodex:test:DummyDummyDummyDummyDummyDummyDummyDummyDummyD"
)

json_response = get(f"/identifiers/{destination_did}")

print("The response is as follows.\n")
print(json_response)
