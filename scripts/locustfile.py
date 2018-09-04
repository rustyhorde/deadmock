from locust import HttpLocust, TaskSet, task

class UserBehavior(TaskSet):
    @task(1)
    def healthcheck(self):
        self.client.get("/mobilecheckout/healthcheck")

    @task(1)
    def cards(self):
        self.client.get("/mobilecheckout/api/v1/card/b14d93bd-67c2-aa1b-c973-f6b084403e3e")

    @task(10)
    def plaintext(self):
        self.client.get("/plaintext")

    @task(10)
    def json(self):
        self.client.get("/json")

    @task(1)
    def jozias(self):
        self.client.get("/")

class WebsiteUser(HttpLocust):
    task_set = UserBehavior
    min_wait = 1000
    max_wait = 2000
