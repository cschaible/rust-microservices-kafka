package simulations

import actions.createAccommodation
import config.Configuration
import io.gatling.javaapi.core.CoreDsl.*
import io.gatling.javaapi.core.Simulation

class CreateAccommodationSimulation : Simulation() {

  private val configuration = Configuration(3005)

  init {

    val concurrentUsers = configuration.concurrentUsers
    val rampUpDuration = configuration.rampUpDuration
    val testDuration = configuration.testDuration

    setUp(
        scenario("Create Accommodation")
            .createAccommodation()
            .injectClosed(
                rampConcurrentUsers(1).to(concurrentUsers).during(rampUpDuration),
                constantConcurrentUsers(concurrentUsers).during(testDuration))
            .protocols(configuration.httpProtocol))
  }
}
