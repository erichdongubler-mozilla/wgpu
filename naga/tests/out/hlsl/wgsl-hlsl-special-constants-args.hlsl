struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b0, space1);

struct VIn {
    uint vi : SV_VertexID;
    uint ii : SV_InstanceID;
};

struct CsIn {
    uint3 n : SV_GroupID;
    uint lii : SV_GroupIndex;
};

struct VertexInput_vs_helper {
    uint ii : SV_InstanceID;
    uint vi : SV_VertexID;
};

struct VertexInput_vs_let {
    uint ii_1 : SV_InstanceID;
    uint vi_1 : SV_VertexID;
};

struct ComputeInput_cs {
    uint3 n_2 : SV_GroupID;
};

struct ComputeInput_cs_struct {
    uint lii : SV_GroupIndex;
    uint3 n_3 : SV_GroupID;
};

uint helper(VIn v_2)
{
    return (v_2.ii + v_2.vi);
}

uint helper_wg(uint3 n_1)
{
    return n_1.x;
}

uint helper_cs_in(CsIn i_1)
{
    return (i_1.n.x + i_1.lii);
}

float4 vs_helper(VertexInput_vs_helper vertexinput_vs_helper) : SV_Position
{
    VIn v = { (_NagaConstants.first_vertex + vertexinput_vs_helper.vi), (_NagaConstants.first_instance + vertexinput_vs_helper.ii) };
    const uint _e1 = helper(v);
    return (float(_e1)).xxxx;
}

float4 vs_let(VertexInput_vs_let vertexinput_vs_let) : SV_Position
{
    VIn v_1 = { (_NagaConstants.first_vertex + vertexinput_vs_let.vi_1), (_NagaConstants.first_instance + vertexinput_vs_let.ii_1) };
    uint ii_2 = v_1.ii;
    uint a = (ii_2 + 1u);
    uint b = (ii_2 + 2u);
    return float4(float(a), float(b), 0.0, 1.0);
}

[numthreads(1, 1, 1)]
void cs(ComputeInput_cs computeinput_cs)
{
    uint3 n = uint3(_NagaConstants.first_vertex, _NagaConstants.first_instance, _NagaConstants.other);
    const uint _e1 = helper_wg(n);
    return;
}

[numthreads(1, 1, 1)]
void cs_struct(ComputeInput_cs_struct computeinput_cs_struct)
{
    CsIn i = { uint3(_NagaConstants.first_vertex, _NagaConstants.first_instance, _NagaConstants.other), computeinput_cs_struct.lii };
    const uint _e1 = helper_cs_in(i);
    return;
}
