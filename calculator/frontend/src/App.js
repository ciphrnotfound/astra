// Update React frontend to make a POST request to the Go backend API

function handleSubmit = async (event) => {
    event.preventDefault();
    const response = await axios.post('/api/calculate', {
        num1: number1,
        num2: number2
    });
    setResult(response.data.result);
};