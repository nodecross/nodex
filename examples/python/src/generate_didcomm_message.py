from sock import post
import json
import pprint


# PLEASE WRITE destination_did, message, AND operation_tag.
destination_did = (
    "did:nodex:test:DummyDummyDummyDummyDummyDummyDummyDummyDummyD"
)
message = {
    "message": {
        "string": "value",
        "number": 1,
        "boolean": True,
        "array": ["foo", "bar", "baz"],
        "map": {"key": "value"},
    }
}
operation_tag = "test-operation-tag"


payload = {
    "destination_did": destination_did,
    "message": json.dumps(message),
    "operation_tag": operation_tag,
}

json_response = post("/create-didcomm-message", payload)

print("The response is as follows.\n")
print(json_response)

print('\nPlease paste below to "verify_didcomm_message.py".\n')
pprint.pprint(json.loads(json_response), sort_dicts=False)
