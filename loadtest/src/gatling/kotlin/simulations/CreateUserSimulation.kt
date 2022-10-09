package simulations

import actions.createUser
import config.Configuration
import io.gatling.javaapi.core.CoreDsl.*
import io.gatling.javaapi.core.Simulation

class CreateUserSimulation : Simulation() {

  private val configuration = Configuration(3000)

  init {

    val concurrentUsers = configuration.concurrentUsers
    val rampUpDuration = configuration.rampUpDuration
    val testDuration = configuration.testDuration

    setUp(
        scenario("Create User")
            .createUser()
            .injectClosed(
                rampConcurrentUsers(1).to(concurrentUsers).during(rampUpDuration),
                constantConcurrentUsers(concurrentUsers).during(testDuration))
            .protocols(configuration.httpProtocol))
  }
}
