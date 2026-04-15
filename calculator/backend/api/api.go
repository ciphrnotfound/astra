// API endpoint for calculator operations

import "net/http"

func Calculator(api *API) {
    api.HandleFunc("/calculate", func(w http.ResponseWriter, r *http.Request) {
        num1, _ := strconv.Atoi(r.URL.Query().Get("num1"))
        num2, _ := strconv.Atoi(r.URL.Query().Get("num2"))
        result := num1 + num2
        w.Write([]byte{"result": result})
    })
}