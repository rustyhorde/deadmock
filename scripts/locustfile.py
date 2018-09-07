from locust import HttpLocust, TaskSet, task

class UserBehavior(TaskSet):
    @task(1)
    def proxy_healthcheck(self):
        self.client.get("/mobilecheckout/healthcheck", name = "Proxy - Healthcheck")

    @task(1)
    def proxy_get_card(self):
        self.client.get("/mobilecheckout/api/v1/card/b14d93bd-67c2-aa1b-c973-f6b084403e3e", name = "Proxy - Get Card")

    @task(1)
    def proxy_jasonoziascom(self):
        self.client.get("/", name = "Proxy - jasonozias.com")

    @task(10)
    def get_card_visa_static(self):
        self.client.get("/mobilecheckout/api/v1/card",
            headers = { "X-Card-Type": "VS" },
            name = "Static - Get Card (Visa)")

    @task(10)
    def get_card_mastercard_static(self):
        self.client.get("/mobilecheckout/api/v1/card",
            headers = { "X-Card-Type": "MC" },
            name = "Static - Get Card (Mastercard)")

    @task(10)
    def plaintext_static(self):
        self.client.get("/plaintext", name = "Static - Plaintext")

    @task(10)
    def json_static(self):
        self.client.get("/json", name = "Static - JSON")

class WebsiteUser(HttpLocust):
    task_set = UserBehavior
    min_wait = 1000
    max_wait = 2000
