export default {
  interfaces_handler: function(data, state) {
    state.interfaces = data
  },
  status_handler: function(data, state) {
    state.services = data
  },
  wifi_scan_result_handler: function(data, state) {
    state.wifiScan = data
  },
  wifi_status_handler: function(data, state) {
    state.wifiStatus = data
  },
  log_handler: function(data, state) {
    var svc_name = data.service
    var svc_detail = state.service[svc_name]
    if (svc_detail) {
      svc_detail.log.push(data.message)
      while (svc_detail.log.length > 100) {
        svc_detail.log.unshift()
      }
    }
  },
}
