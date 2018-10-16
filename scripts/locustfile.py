from locust import HttpLocust, TaskSet, task

class UserBehavior(TaskSet):
    @task(1)
    def proxy_healthcheck(self):
        self.client.get("/mobilecheckout/healthcheck",
            name = "Exact Match (Method, URL, Proxy, healthcheck)")

    @task(1)
    def proxy_get_card(self):
        self.client.get("/mobilecheckout/api/v1/card/b14d93bd-67c2-aa1b-c973-f6b084403e3e",
            name = "Exact Match (URL, Method, Proxy, card")

    @task(1)
    def proxy_jasonoziascom(self):
        self.client.get("/",
            name = "Exact Match (URL, Method, Proxy, jasonozias.com)")

    @task(10)
    def get_card_visa_static(self):
        self.client.get("/mobilecheckout/api/v1/card",
            headers = { "X-Card-Type": "VS" },
            name = "Exact Match (URL, Method, Header, Visa)")

    @task(10)
    def get_card_mastercard_static(self):
        self.client.get("/mobilecheckout/api/v1/card",
            headers = { "X-Card-Type": "MC" },
            name = "Exact Match (URL, Method, Header, Mastercard)")

    @task(10)
    def plaintext_static(self):
        self.client.get("/plaintext",
            name = "Exact Match (URL, plaintext)")

    @task(10)
    def json_static(self):
        self.client.get("/json",
            name = "Exact Match (URL, json)")

    @task(10)
    def json_static_not_found(self):
        with self.client.get("/json",
            headers = { "X-Correlation-Id": "not-found" },
            name = "Exact Match (URL, Header)",
            catch_response = True) as response:
            if response.status_code == 404:
                response.success()

    @task(10)
    def weather_static(self):
        self.client.get("/weather/45039", name = "Pattern Match (URL)")

    @task(10)
    def pattern_match_header(self):
        self.client.get("/header-pattern",
            headers = { "X-Pattern-Match": "yoda-bloda" },
            name = "Pattern Match (Header)")

    @task(10)
    def post_static(self):
        self.client.post("/method-pattern",
            name = "Static - Method Pattern (POST)")

    @task(10)
    def put_static(self):
        self.client.put("/method-pattern",
            name = "Static - Method Pattern (PUT)")

    @task(10)
    def patch_static(self):
        self.client.patch("/method-pattern",
            name = "Static - Method Pattern (PATCH)")

class WebsiteUser(HttpLocust):
    task_set = UserBehavior
    min_wait = 1000
    max_wait = 2000
