import requests
import json
import time
import os

# Configuration
BASE_URL_DOC = "http://localhost:3001"
BASE_URL_LC = "http://localhost:3002"
BASE_URL_CHAIN = "http://localhost:3004"

# Mock JWT Tokens (In a real scenario, these would be signed properly)
# For our dev environment, we might need to adjust the Auth middleware to accept these or use a helper to generate them.
# Assuming the services are running with a simple JWT secret "supersecretkey"
import jwt
import datetime

SECRET = "supersecretkey"

def generate_token(user_id, role):
    payload = {
        "sub": user_id,
        "role": role,
        "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=1)
    }
    return jwt.encode(payload, SECRET, algorithm="HS256")

IMPORTER_ID = "00000000-0000-0000-0000-000000000001"
EXPORTER_ID = "00000000-0000-0000-0000-000000000002"
ISSUING_BANK_ID = "00000000-0000-0000-0000-000000000003"
ADVISING_BANK_ID = "00000000-0000-0000-0000-000000000004"


def print_step(step):
    print(f"\n{'='*20} {step} {'='*20}")

def test_upload_document():
    print_step("1. Upload Document")
    
    # Create a dummy file
    with open("test_doc.txt", "w") as f:
        f.write("This is a test document for LC digitization.")

    url = f"{BASE_URL_DOC}/documents"
    headers = {"Authorization": f"Bearer {TOKEN_IMPORTER}"}
    files = {'file': open('test_doc.txt', 'rb')}

    try:
        response = requests.post(url, headers=headers, files=files)
        response.raise_for_status()
        data = response.json()
        print("Success:", json.dumps(data, indent=2))
        return data['document_id']
    except Exception as e:
        print(f"Failed: {e}")
        if 'response' in locals():
            print(response.text)
        return None

def test_create_lc(doc_id):
    print_step("2. Create Letter of Credit")
    if not doc_id:
        print("Skipping LC creation due to missing doc_id")
        return None

    url = f"{BASE_URL_LC}/lc"
    headers = {
        "Authorization": f"Bearer {TOKEN_IMPORTER}",
        "Content-Type": "application/json"
    }
    payload = {
        "beneficiary_id": EXPORTER_ID,
        "issuing_bank_id": ISSUING_BANK_ID,
        "advising_bank_id": ADVISING_BANK_ID,
        "amount": "50000.00",
        "currency": "USD",
        "expiry_date": "2024-12-31T00:00:00Z",
        "document_ids": [doc_id]
    }

    try:
        response = requests.post(url, headers=headers, json=payload)
        response.raise_for_status()
        data = response.json()
        print("Success:", json.dumps(data, indent=2))
        return data['id']
    except Exception as e:
        print(f"Failed: {e}")
        if 'response' in locals():
            print(response.text)
        return None

def test_approve_lc(lc_id):
    print_step("3. Submit & Approve LC")
    if not lc_id:
        print("Skipping approval due to missing lc_id")
        return

    # 1. Submit (Importer)
    print("-> Submitting LC (Importer)...")
    url = f"{BASE_URL_LC}/lc/{lc_id}/status"
    headers = {
        "Authorization": f"Bearer {TOKEN_IMPORTER}",
        "Content-Type": "application/json"
    }
    try:
        requests.put(url, headers=headers, json={"status": "Submitted"}).raise_for_status()
        print("Submitted.")
    except Exception as e:
        print(f"Submit Failed: {e}")
        return

    # 2. Start Review (Issuing Bank)
    print("-> Starting Review (Issuing Bank)...")
    headers["Authorization"] = f"Bearer {TOKEN_ISSUING_BANK}"
    try:
        requests.put(url, headers=headers, json={"status": "Review"}).raise_for_status()
        print("Review Started.")
    except Exception as e:
        print(f"Review Failed: {e}")
        return

    # 3. Approve (Issuing Bank)
    print("-> Approving (Issuing Bank)...")
    try:
        requests.put(url, headers=headers, json={"status": "Approved"}).raise_for_status()
        print("Approved.")
    except Exception as e:
        print(f"Approval Failed: {e}")
        return



if __name__ == "__main__":
    print("Starting Test Flow...")
    # Ensure dependencies are installed
    # pip install requests pyjwt
    
    doc_id = test_upload_document()
    lc_id = test_create_lc(doc_id)
    test_approve_lc(lc_id)

    print("\nTest Flow Complete.")
