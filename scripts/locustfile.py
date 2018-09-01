from locust import HttpLocust, TaskSet, task

class UserBehavior(TaskSet):
    @task(1)
    def healthcheck(self):
        self.client.get("/mobilecheckout/healthcheck")

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
