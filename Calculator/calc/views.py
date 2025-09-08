from django.shortcuts import render
from math import pow

def calculator(request):
    result1 = None  # Result for (1 - ((1 - qF)^L * (1 - qA)^L))^K - (1 - ((1 - qF)^L))^K
    result2 = None  # Result for (1 - (1 - qF)^L)^K
    result3 = None  # Result for 1 - (1 - ((1 - qF)^L * qA^L))^K
    error_message = None

    # Default values for inputs
    qF = request.POST.get('qF', '')
    qA = request.POST.get('qA', '')
    L = request.POST.get('L', '')
    K = request.POST.get('K', '')

    if request.method == 'POST':
        try:
            qF = float(qF)  # Get qF from the form
            qA = float(qA)  # Get qA from the form
            L = float(L)    # Get L from the form
            K = float(K)    # Get K from the form

            # Validate qF and qA are between 0 and 1
            if qF < 0 or qF > 1 or qA < 0 or qA > 1:
                error_message = "Error: qF and qA must be between 0 and 1."
            # Validate L is an integer greater than or equal to 2
            elif not (L.is_integer() and L >= 2):
                error_message = "Error: L must be an integer greater than or equal to 2."
            # Validate K is an integer greater than or equal to 1
            elif not (K.is_integer() and K >= 1):
                error_message = "Error: K must be an integer greater than or equal to 1."
            else:
                # Convert L and K to integers
                L = int(L)
                K = int(K)

                # Compute (1 - (1 - qF)^L)^K
                term1 = pow((1 - qF), L)  # (1 - qF)^L
                result2 = pow(1 - term1, K)  # (1 - (1 - qF)^L)^K

                # Compute (1 - ((1 - qF)^L * (1 - qA)^L))^K - (1 - (1 - qF)^L)^K
                term2 = pow((1 - qA), L)  # (1 - qA)^L
                part1 = 1 - (term1 * term2)    # 1 - ((1 - qF)^L * (1 - qA)^L)
                result1 = pow(part1, K) - result2  # Final result

                # Compute 1 - (1 - ((1 - qF)^L * qA^L))^K
                term3 = pow(qA, L)        # qA^L
                part2 = 1 - (term1 * term3)    # 1 - ((1 - qF)^L * qA^L)
                result3 = 1 - pow(part2, K)  # Final result

                # Format results in scientific notation
                result1 = f"{result1:.4e}" if result1 is not None else None
                result2 = f"{result2:.4e}" if result2 is not None else None
                result3 = f"{result3:.4e}" if result3 is not None else None
        except ValueError:
            error_message = "Error: Invalid input. Please enter numeric values."

    return render(request, 'calc/calculator.html', {
        'result1': result1,
        'result2': result2,
        'result3': result3,
        'error_message': error_message,
        'qF': qF,
        'qA': qA,
        'L': int(float(L)) if L != '' else '',  # Ensure L is an integer
        'K': int(float(K)) if K != '' else ''   # Ensure K is an integer
    })