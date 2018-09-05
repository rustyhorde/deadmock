from locust import HttpLocust, TaskSet, task

class UserBehavior(TaskSet):
    @task(1)
    def healthcheck_proxy(self):
        self.client.get("/mobilecheckout/healthcheck", name = "Healthcheck Proxy")

    @task(1)
    def get_card_proxy(self):
        self.client.get("/mobilecheckout/api/v1/card/b14d93bd-67c2-aa1b-c973-f6b084403e3e", name = "Get Card Proxy")

    @task(10)
    def get_card_static(self):
        self.client.get("/mobilecheckout/api/v1/card/b14d93bd-67c2-aa1b-c973-f6b084403e3e",
            headers = { "X-Correlation-Id": "deadmock-default-visa" },
            name = "Get Card Static")

    @task(10)
    def plaintext_static(self):
        self.client.get("/plaintext", name = "Static Plaintext")

    @task(10)
    def json_static(self):
        self.client.get("/json", name = "Static JSON")

    @task(1)
    def jasonozias_proxy(self):
        self.client.get("/", name = "jasonozias.com Proxy")

class WebsiteUser(HttpLocust):
    task_set = UserBehavior
    min_wait = 1000
    max_wait = 2000
